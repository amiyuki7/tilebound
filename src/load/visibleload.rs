use super::*;

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut spawn_entity_event: EventWriter<SpawnEntityEvent>,
) {
    commands.insert_resource(ClearColor(Color::ALICE_BLUE));
    // Floor
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(50.0).into()),
            material: materials.add(Color::LIME_GREEN.into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(Name::new("Floor"));

    spawn_entity_event.send(SpawnEntityEvent {
        entity_type: REntityType::Kraug,
        is_player: true,
    });
}

pub fn scene_is_setup(mut commands: Commands) {}
