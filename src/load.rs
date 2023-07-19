use crate::*;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .init_resource::<LoadingAssets>()
            .insert_resource(ClearColor(Color::RED))
            .add_startup_system(asset_loading)
            .add_system(
                check_assets_ready
                    .after(asset_loading)
                    .in_set(OnUpdate(GameState::Loading)),
            )
            .add_system((loading_phase_two).in_set(OnUpdate(GameState::LoadingPhaseTwo)));
    }
}

/// Contains a list of handles in the process of loading
#[derive(Resource, Default)]
struct LoadingAssets(Vec<HandleUntyped>);

/// Custom metadata interface for an AnimationClip
pub struct AnimationMeta {
    pub handle: Handle<AnimationClip>,
    pub duration: f32,
}

impl AnimationMeta {
    fn new(handle: Handle<AnimationClip>) -> Self {
        Self { handle, duration: 0.0 }
    }
}

pub struct REntityMeta {
    pub scene: Handle<Scene>,
    /// \ 0 => Equip
    /// \ 1 => Disarm
    /// \ 2 => Idle Armed
    /// \ 3 => Idle Unarmed
    /// \ 4 => Attack
    /// \ 5 => Skill A
    /// \ 6 => Skill B
    /// \ 7 => Death
    /// \ 8 => Move
    /// \ 9 => Run
    /// 10 => Interact
    pub animations: Vec<AnimationMeta>,
    pub weapon_scene: Handle<Scene>,
    pub weapon_transform: Transform,
}

#[derive(Reflect, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum REntityType {
    Kraug,
}

#[derive(Resource)]
pub struct REntityMap(pub HashMap<REntityType, REntityMeta>);

#[derive(Component)]
struct LoadingCameraMarker;

/// Loads primary asset data into the [`REntityMap`](tilebound::load::REntityMap) resource.
fn asset_loading(mut commands: Commands, asset_server: Res<AssetServer>, mut loading: ResMut<LoadingAssets>) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(Name::new("loading_screen_camera"))
        .insert(LoadingCameraMarker);

    let mut r_entity_map = HashMap::new();

    let kraug_handle: Handle<Scene> = asset_server.load("kraug.glb#Scene0");
    loading.0.push(kraug_handle.clone_untyped());

    let kraug_animations: Vec<_> = (1..=11)
        .map(|idx| {
            let handle: Handle<AnimationClip> = asset_server.load(format!("kraug.glb#Animation{idx}"));
            loading.0.push(handle.clone_untyped());

            // handle
            AnimationMeta::new(handle)
        })
        .collect();

    let axe_handle: Handle<Scene> = asset_server.load("viking_axe.glb#Scene0");
    loading.0.push(axe_handle.clone_untyped());

    let axe_transform = Transform::from_scale(Vec3::splat(1.5))
        .with_translation(Vec3::new(20.9, -15.2, -5.6))
        .with_rotation(Quat::from_rotation_x(1.817) * Quat::from_rotation_y(5.417) * Quat::from_rotation_z(5.266));

    r_entity_map.insert(
        REntityType::Kraug,
        REntityMeta {
            scene: kraug_handle,
            animations: kraug_animations,
            weapon_scene: axe_handle,
            weapon_transform: axe_transform,
        },
    );

    commands.insert_resource(REntityMap(r_entity_map));
}

/// Assets may take time to load. Utilises [`LoadingAssets`](tilebound::load::LoadingAssets) to track progress of loaded
/// assets. Manages [`GameState`](tilebound::GameState).
fn check_assets_ready(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loading: Res<LoadingAssets>,
    mut next_game_state: ResMut<NextState<GameState>>,
    camera_2d: Query<Entity, With<LoadingCameraMarker>>,
) {
    match asset_server.get_group_load_state(loading.0.iter().map(|handle| handle.id())) {
        state @ bevy::asset::LoadState::Loading => {
            debug!("{state:?}");
        }
        state @ bevy::asset::LoadState::Loaded => {
            debug!("{state:?}");
            commands.remove_resource::<LoadingAssets>();

            commands.entity(camera_2d.get_single().unwrap()).despawn_recursive();

            next_game_state.set(GameState::LoadingPhaseTwo);
        }
        bevy::asset::LoadState::Failed => {
            panic!("Asset loaing failed");
        }
        _ => {}
    }
}

/// Other loading priorities including - associating animations with correct duration metadata. Manages
/// [`GameState`](tilebound::GameState)
fn loading_phase_two(
    animations: Res<Assets<AnimationClip>>,
    mut re_map: ResMut<REntityMap>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for (_, meta) in re_map.0.iter_mut() {
        let anims = &mut meta.animations;

        for (idx, anim) in anims.iter_mut().enumerate() {
            if let Some(clip) = animations.get(&anim.handle) {
                let duration = clip.duration();

                anim.duration = duration;
                debug!("Animation @{idx} dur {duration}");
            }
        }
    }

    if re_map
        .0
        .iter()
        .all(|(_, rmeta)| rmeta.animations.iter().all(|anim| anim.duration != 0.0))
    {
        next_game_state.set(GameState::InGame)
    }
}
