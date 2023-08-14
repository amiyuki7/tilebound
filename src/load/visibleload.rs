use super::*;

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut spawn_entity_event: EventWriter<SpawnEntityEvent>,
) {
    commands.insert_resource(ClearColor(Color::ALICE_BLUE));
    commands.insert_resource(Inventory::default());

    // Lighting to brighten everything up
    commands.insert_resource(AmbientLight {
        color: Color::Rgba {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        },
        brightness: 0.6,
    });

    spawn_entity_event.send(SpawnEntityEvent {
        entity_type: REntityType::Kraug,
        is_player: true,
    });
}

pub fn scene_is_setup(
    player_query: Query<&Player>,
    camera_query: Query<&PlayerCameraMarker>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let player = player_query.get_single();
    let camera = camera_query.get_single();

    if player.is_ok() && camera.is_ok() {
        next_game_state.set(GameState::InGame);
    }
}
