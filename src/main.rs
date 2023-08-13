use std::f32::consts::PI;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    window::WindowMode,
};

use bevy_mod_fbx::FbxPlugin;
use bevy_mod_picking::prelude::*;
use bevy_scene_hook::*;

use tilebound::*;

use bevy_mod_picking::{self, PickableBundle};

use bevy_inspector_egui::bevy_egui;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, StateInspectorPlugin, WorldInspectorPlugin};
use tilebound::map_load::MapContext;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Tilebound".into(),
                        mode: WindowMode::BorderlessFullscreen,
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::TRACE,
                    filter: "wgpu=warn,bevy_ecs=info,winit=info,naga=info,bevy_app=info,bevy_winit=info\
                        ,bevy_render=info,bevy_core=info,gilrs=info,bevy_picking_core=warn"
                        .to_string(),
                }),
        )
        .add_plugin(FbxPlugin)
        .add_plugin(HookPlugin)
        .add_startup_system(reset_world)
        .add_plugins(DefaultPickingPlugins.build().disable::<DefaultHighlightingPlugin>())
        .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        // .add_plugin(EditorPlugin::default())
        .insert_resource(ClearColor(Color::ALICE_BLUE))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.2,
        })
        // .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(LoadingPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(AnimEnginePlugin)
        .add_plugin(InventoryPlugin)
        .add_plugin(StateInspectorPlugin::<GameState>::default())
        .add_plugin(ResourceInspectorPlugin::<MapContext>::default())
        .add_plugin(ResourceInspectorPlugin::<CombatManager>::default())
        // .add_system(spawn_scene.in_schedule(OnEnter(GameState::InGame)))
        .add_system(update_world)
        .add_plugin(StateInspectorPlugin::<GIState>::default())
        .register_type::<Tile>()
        .register_type::<Enemy>()
        .add_systems(
            (
                // update_tile_pos.before(button_reset_system),
                // update_player_pos,
                // button_system,
                // button_reset_system.before(button_system),
                // enemy_ai,
                // update_health_bar,
                // update_enemy_health,
                update_tile_state_stable,
            )
                // .after(spawn_scene)
                .in_set(OnUpdate(GameState::InGame))
                .in_set(OnUpdate(GIState::Unlocked)),
        )
        .insert_resource(MapContext::from_map("1".to_string()))
        .add_plugin(CombatPlugin)
        .add_system(move_player_stable.in_set(OnUpdate(GIState::LockedByMovement)))
        .run();
}

// // TODO: Remove in future commit
// fn spawn_scene(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     asset_server: Res<AssetServer>,
//     mut spawn_entity_sender: EventWriter<SpawnEntityEvent>,
// ) {
//     // !Camera
//     // commands.spawn((
//     //     Camera3dBundle {
//     //         transform: Transform::from_xyz(13.50, 25.0, 20.0).looking_at(Vec3::new(13.50, 0.0, 10.0), Vec3::Y),
//     //         ..Default::default()
//     //     },
//     //     RaycastPickCamera::default(),
//     // ));
//     // !Floor
//     commands.spawn(PbrBundle {
//         mesh: meshes.add(shape::Plane::from_size(50.0).into()),
//         material: materials.add(Color::DARK_GREEN.into()),
//         transform: Transform::from_xyz(15.0, 0.0, 20.0),
//         ..default()
//     });

//     // commands.spawn(PointLightBundle {
//     //     point_light: PointLight {
//     //         intensity: 1500.0,
//     //         shadows_enabled: true,
//     //         ..default()
//     //     },
//     //     transform: Transform::from_xyz(13.50, 10.0, 10.0),
//     //     ..default()
//     // });
//     // Gonna need a lot of point lights otherwise. Looks iffy, but it'll do
//     // !Lighting
//     commands.insert_resource(AmbientLight {
//         color: Color::Rgba {
//             red: 1.0,
//             green: 1.0,
//             blue: 1.0,
//             alpha: 1.0,
//         },
//         brightness: 0.6,
//     });
//     // !Cube idk why this exists
//     commands.spawn(PbrBundle {
//         mesh: meshes.add(shape::Cube::new(1.0 * 2.0).into()),
//         transform: Transform::from_xyz(0.0, -0.8, 0.0),
//         material: materials.add(Color::SALMON.into()),
//         ..default()
//     });

//     // // !Tiles
//     // for tile in load_new_map_data("1".to_string()) {
//     //     commands.spawn((
//     //         PbrBundle {
//     //             mesh: meshes.add(Mesh::from(shape::RegularPolygon {
//     //                 radius: 5.2 * SCALE,
//     //                 sides: 6,
//     //             })),
//     //             material: materials.add(Color::rgba(1.0, 1.0, 1.0, 0.6).into()),
//     //             transform: Transform::from_scale(Vec3::splat(SCALE))
//     //                 .with_translation(Vec3::new(
//     //                     HORIZONTAL_SPACING * tile.coord.q as f32 + tile.coord.r as f32 % 2.0 * HOR_OFFSET,
//     //                     1.0,
//     //                     VERTICAL_SPACING * tile.coord.r as f32,
//     //                 ))
//     //                 .with_rotation(Quat::from_axis_angle(Vec3 { x: 1.0, y: 0.0, z: 0.0 }, -PI / 2.0)),
//     //             ..Default::default()
//     //         },
//     //         tile,
//     //         PickableBundle::default(),
//     //         RaycastPickTarget::default(),
//     //         OnPointer::<Over>::target_component_mut::<Tile>(|_, tile| tile.is_hovered = true),
//     //         OnPointer::<Out>::target_component_mut::<Tile>(|_, tile| tile.is_hovered = false),
//     //         OnPointer::<Click>::target_component_mut::<Tile>(|_, tile| {
//     //             if tile.can_be_clicked {
//     //                 if tile.is_clicked {
//     //                     tile.is_clicked = false
//     //                 } else {
//     //                     tile.is_clicked = true
//     //                 }
//     //             }
//     //         }),
//     //     ));
//     //     // .with_children(|parent| {
//     //     //     parent.spawn(SceneBundle {
//     //     //         scene: asset_server.load("tile.glb#Scene0"),
//     //     //         transform: Transform::from_translation(Vec3::new(0.0, 0.0, -0.15))
//     //     //             .with_rotation(Quat::from_axis_angle(Vec3 { x: 1.0, y: 0.0, z: 0.0 }, -PI / 2.0)),
//     //     //         ..default()
//     //     //     });
//     //     // });
//     // }

//     // !Enemies
//     // commands.spawn((
//     //     PbrBundle {
//     //         mesh: meshes.add(Mesh::from(shape::Capsule {
//     //             radius: 1.2 * SCALE,
//     //             rings: 1,
//     //             depth: 1.0,
//     //             ..default()
//     //         })),
//     //         material: materials.add(Color::rgb(1.0, 0.5, 0.5).into()),
//     //         transform: Transform::from_xyz(
//     //             HORIZONTAL_SPACING * 6f32 + 3f32 % 2.0 * HOR_OFFSET,
//     //             2.5,
//     //             VERTICAL_SPACING * 3f32,
//     //         ),
//     //         ..default()
//     //     },
//     //     Enemy::new(6, 3, 3, 2, 10.0, 20.0),
//     // ));

//     // !Player
//     spawn_entity_sender.send(SpawnEntityEvent {
//         entity_type: REntityType::Kraug,
//         is_player: true,
//     });

//     commands
//         .spawn(PbrBundle {
//             mesh: meshes.add(Mesh::from(shape::UVSphere {
//                 radius: 2.0 * SCALE,
//                 sectors: 8,
//                 stacks: 8,
//             })),
//             material: materials.add(Color::rgb(1.0, 0.7, 0.4).into()),
//             transform: Transform::from_xyz(0.0, 2.0, 0.0),
//             ..default()
//         })
//         .insert(PlayerAction::SpellCast(SpellType::Fireball));

//     // !Ui
//     let fireball_icon = asset_server.load("2D/Fireball.png");
//     let movement_icon = asset_server.load("2D/Running.png");
//     commands
//         .spawn(NodeBundle {
//             style: Style {
//                 size: Size {
//                     width: Val::Percent(100.0),
//                     height: Val::Px(128.0),
//                 },
//                 align_items: AlignItems::Center,
//                 justify_content: JustifyContent::Center,
//                 ..default()
//             },
//             ..default()
//         })
//         .with_children(|parent| {
//             //Fireball Button
//             parent
//                 .spawn(ButtonBundle {
//                     style: Style {
//                         size: Size::new(Val::Px(128.0), Val::Px(128.0)),
//                         // horizontally center child text
//                         justify_content: JustifyContent::Center,
//                         // vertically center child text
//                         align_items: AlignItems::Center,
//                         position_type: PositionType::Absolute,
//                         position: UiRect {
//                             right: Val::Px(0.0),
//                             top: Val::Px(0.0),
//                             ..default()
//                         },
//                         ..default()
//                     },
//                     background_color: NORMAL_BUTTON.into(),
//                     image: UiImage::new(fireball_icon),
//                     ..default()
//                 })
//                 .insert(ButtonType::Spell(SpellType::Fireball))
//                 .insert(ToggleButton::new())
//                 .with_children(|parent| {
//                     parent
//                         .spawn(TextBundle::from_section(
//                             "Fireball",
//                             TextStyle {
//                                 font: asset_server.load("fonts/FiraSans-Bold.ttf"),
//                                 font_size: 40.0,
//                                 color: Color::rgb(0.9, 0.9, 0.9),
//                             },
//                         ))
//                         .insert(ButtonText {
//                             active_text: "Casting".to_string(),
//                             passive_text: "Fireball".to_string(),
//                         });
//                 });
//             // Movement button
//             parent
//                 .spawn(ButtonBundle {
//                     style: Style {
//                         size: Size::new(Val::Px(128.0), Val::Px(128.0)),
//                         // horizontally center child text
//                         justify_content: JustifyContent::Center,
//                         // vertically center child text
//                         align_items: AlignItems::Center,
//                         position_type: PositionType::Absolute,
//                         position: UiRect {
//                             right: Val::Px(128.0),
//                             top: Val::Px(0.0),
//                             ..default()
//                         },
//                         ..default()
//                     },
//                     background_color: NORMAL_BUTTON.into(),
//                     image: UiImage::new(movement_icon),
//                     ..default()
//                 })
//                 .insert(ButtonType::Movement)
//                 .insert(ToggleButton::new())
//                 .with_children(|parent| {
//                     parent
//                         .spawn(TextBundle::from_section(
//                             "Move",
//                             TextStyle {
//                                 font: asset_server.load("fonts/FiraSans-Bold.ttf"),
//                                 font_size: 40.0,
//                                 color: Color::rgb(0.9, 0.9, 0.9),
//                             },
//                         ))
//                         .insert(ButtonText {
//                             active_text: "Moving".to_string(),
//                             passive_text: "Move".to_string(),
//                         });
//                 });
//             // Debug text
//             parent
//                 .spawn(TextBundle::from_section(
//                     "Something",
//                     TextStyle {
//                         font: asset_server.load("fonts/FiraSans-Bold.ttf"),
//                         font_size: 40.0,
//                         color: Color::rgb(0.9, 0.9, 0.9),
//                     },
//                 ))
//         });
//     // Healthbar, separate node so that it can be moved to a better position
//     let health = Health::new(100.);
//     commands
//         .spawn(NodeBundle {
//             style: Style {
//                 size: Size::new(Val::Px(200.0), Val::Px(50.0)),
//                 border: UiRect::all(Val::Px(2.0)),
//                 position_type: PositionType::Absolute,
//                 // position: UiRect::new(Val::Px(0.0), Val::Px(200.0), Val::Px(0.0), Val::Px(100.0)),
//                 ..Default::default()
//             },
//             background_color: Color::GRAY.into(),
//             ..Default::default()
//         })
//         .with_children(|parent| {
//             parent
//                 .spawn(NodeBundle {
//                     style: Style {
//                         size: Size::new(Val::Percent((health.hp / health.max_hp) * 100.0), Val::Percent(100.0)),
//                         ..Default::default()
//                     },
//                     background_color: Color::GREEN.into(),
//                     ..Default::default()
//                 })
//                 .insert(HealthBar);
//         });

//     commands.insert_resource(health);
// }

#[derive(Component)]
struct AxeAnchor;
