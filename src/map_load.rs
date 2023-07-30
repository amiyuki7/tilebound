use std::{f32::consts::PI, fs};

use bevy_mod_picking::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_inspector_egui::prelude::*;

use crate::*;
#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct MapContext {
    pub id: String,
    pub load_new_region: bool,
    pub current_map: Region,
}
impl MapContext {
    pub fn from_map(id: String) -> Self {
        MapContext {
            id: id.clone(),
            load_new_region: true,
            current_map: load_new_map_data(id),
        }
    }
    // fn fetch(id: &str) -> Vec<Tile>
}

#[derive(Serialize, Deserialize, Reflect, Default)]
pub struct Region {
    pub tiles: Vec<Tile>,
    pub enemies: Option<Vec<Enemy>>,
}

pub fn load_new_map_data(id: String) -> Region {
    let contents = fs::read_to_string("src/world.json").expect("Something went wrong reading the file");
    let mut deserialized: HashMap<String, Region> = serde_json::from_str(&contents).unwrap();

    deserialized.remove(&id).unwrap()
}

pub fn update_world(
    mut commands: Commands,
    mut map_context: ResMut<MapContext>,
    mut combat_manager: ResMut<CombatManager>,
    tiles_query: Query<Entity, With<Tile>>,
    enemies_query: Query<Entity, With<Enemy>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut debug_text_query: Query<&mut Text, With<DebugText>>,
) {
    if map_context.load_new_region {
        map_context.load_new_region = false;
        for tile in &tiles_query {
            commands.entity(tile).despawn()
        }
        for enemy in &enemies_query {
            commands.entity(enemy).despawn()
        }
        let region = load_new_map_data(map_context.id.clone());
        for tile in region.tiles {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::RegularPolygon {
                        radius: 5.2 * SCALE,
                        sides: 6,
                    })),
                    material: materials.add(Color::rgba(1.0, 1.0, 1.0, 0.6).into()),
                    transform: Transform::from_scale(Vec3::splat(SCALE))
                        .with_translation(Vec3::new(
                            HORIZONTAL_SPACING * tile.coord.q as f32 + tile.coord.r as f32 % 2.0 * HOR_OFFSET,
                            1.0,
                            VERTICAL_SPACING * tile.coord.r as f32,
                        ))
                        .with_rotation(Quat::from_axis_angle(Vec3 { x: 1.0, y: 0.0, z: 0.0 }, -PI / 2.0)),
                    ..Default::default()
                },
                tile,
                PickableBundle::default(),
                RaycastPickTarget::default(),
                OnPointer::<Over>::target_component_mut::<Tile>(|_, tile| tile.is_hovered = true),
                OnPointer::<Out>::target_component_mut::<Tile>(|_, tile| tile.is_hovered = false),
                OnPointer::<Click>::target_component_mut::<Tile>(|_, tile| {
                    if tile.can_be_clicked {
                        if tile.is_clicked {
                            tile.is_clicked = false
                        } else {
                            tile.is_clicked = true
                        }
                    }
                }),
            ));
        }
        if let Some(enemies) = region.enemies {
            combat_manager.in_combat = true;
            for enemy in enemies {
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule {
                            radius: 1.2 * SCALE,
                            rings: 1,
                            depth: 1.0,
                            ..default()
                        })),
                        material: materials.add(Color::rgb(1.0, 0.5, 0.5).into()),
                        transform: Transform::from_xyz(
                            HORIZONTAL_SPACING * enemy.hex_coord.q as f32 + enemy.hex_coord.r as f32 % 2.0 * HOR_OFFSET,
                            2.5,
                            VERTICAL_SPACING * enemy.hex_coord.r as f32,
                        ),
                        ..default()
                    },
                    enemy,
                ));
            }
        } else {
            combat_manager.in_combat = false
        }
        if !debug_text_query.is_empty() {
            let mut debug_text = debug_text_query.single_mut();
            debug_text.sections[0].value = format!(
                "Current world: {}, In combat: {}",
                map_context.id.clone(),
                combat_manager.in_combat
            );
        }
    }
}
