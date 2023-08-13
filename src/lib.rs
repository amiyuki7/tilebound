use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_mod_picking::prelude::RaycastPickCamera;
use bevy_scene_hook::{HookedSceneBundle, SceneHook};

pub mod animengine;
pub mod astar;
pub mod character_creation;
pub mod chest;
pub mod combat;
pub mod inventory;
pub mod load;
pub mod map_load;
pub mod tempui;

pub use animengine::*;
pub use astar::*;
pub use character_creation::*;
pub use chest::*;
pub use combat::*;
pub use inventory::*;
pub use load::*;
pub use map_load::*;
use serde::{Deserialize, Serialize};
pub use tempui::*;

#[derive(States, Reflect, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum GameState {
    #[default]
    /// Load assets
    Loading,
    /// Do extra setup such as determining animation clip durations
    LoadingPhaseTwo,
    Menu,
    // Only happens if a new game is begun
    CharacterCreation,
    /// TODO: Work with VisibleLoading state after LoadingPhaseTwo
    /// Spawning and parent/child hierarchy setup for entities and cameras
    VisibleLoading,
    /// Player controls ready
    InGame,
}

#[derive(States, Reflect, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum UIState {
    Inventory,
    OpenChest,
    #[default]
    Null,
}

#[derive(Component, Reflect, Serialize, Deserialize)]
pub struct Player {
    pub hex_coord: HexCoord,
    pub path: Option<Vec<HexCoord>>,
    pub health: Health,
    pub respawn_point: RespawnPoint,
    // Used for force ending combat phase if player is out of movement
    pub remaining_speed: i32,
    #[serde(default, skip)]
    pub move_timer: Timer,
    pub stats: Stats,
}

impl Player {
    /// `move_timer_duration`: The duration of the running animation of the REntity
    /// Access like so:
    /// ```
    /// fn my_system(re_map: Res<REntityMap>) {
    ///     let run_dur = re_map.0.get(&REntityType::Kraug).unwrap().animations[9].duration;
    /// }
    /// ```
    pub fn new(q: i32, r: i32, move_timer_duration: f32, stats: (i32, i32, i32)) -> Player {
        // Set it close to the just finished amount so we can snap straight into animations during movement
        let mut timer = Timer::from_seconds(move_timer_duration, TimerMode::Repeating);
        timer.set_elapsed(::std::time::Duration::from_secs_f32(move_timer_duration - 0.05));

        Player {
            hex_coord: HexCoord::new(q, r),
            path: None,
            // move_timer: Timer::from_seconds(move_timer_duration, TimerMode::Repeating),
            move_timer: timer,
            health: Health::new(((stats.2 + 5) * 10) as f32),
            respawn_point: RespawnPoint {
                world: "1".to_string(),
                coord: HexCoord::new(0, 0),
            },
            remaining_speed: stats.0,
            stats: Stats {
                speed: stats.0,
                damage: stats.1,
                health: stats.2,
            },
        }
    }

    /// "Resetting" the move timer is setting the elapsed to a split second before just_finished()
    /// triggers, as we want movement animations to run ASAP
    pub fn reset_move_timer(&mut self) {
        self.move_timer
            .set_elapsed(self.move_timer.duration() - ::std::time::Duration::from_secs_f32(0.05));
    }
}

#[derive(Component, Resource, Serialize, Deserialize, Reflect, FromReflect, Clone, Debug)]
pub struct Health {
    pub max_hp: f32,
    pub hp: f32,
}
impl Health {
    pub fn new(hp: f32) -> Health {
        Health { max_hp: hp, hp }
    }
}

#[derive(Component)]
pub struct HealthBar;

// #[derive(PartialEq, Component, Clone, Reflect, FromReflect, Debug)]
// pub enum PlayerAction {
//     Movement,
//     Action(AcitonType),
// }

#[derive(Component, PartialEq)]
pub enum Spell {
    Fireball,
}

pub const SCALE: f32 = 0.54;
pub const HORIZONTAL_SPACING: f32 = 5.2 * SCALE;
pub const VERTICAL_SPACING: f32 = 4.5 * SCALE;
pub const HOR_OFFSET: f32 = 2.6 * SCALE;

#[derive(States, Reflect, PartialEq, Eq, Debug, Clone, Copy, Hash, Default)]
/// Global Interaction State
pub enum GIState {
    /// Disable raycasting
    Locked,
    LockedByMovement,
    #[default]
    /// Enanble raycasting
    Unlocked,
}

pub struct GlobalInteractionLockEvent(GIState);

pub fn change_gi_state(
    mut commands: Commands,
    mut gi_lock_event: EventReader<GlobalInteractionLockEvent>,
    mut next_gi_state: ResMut<NextState<GIState>>,
    raycast_camera: Query<Entity, With<RaycastPickCamera>>,
    player_camera: Query<Entity, With<PlayerCameraMarker>>,
    mut player: Query<&mut Player>,
) {
    let raycast_camera = raycast_camera.get_single();
    let player_camera = player_camera.get_single();

    for event in gi_lock_event.iter() {
        match event.0 {
            // Disable raycasting
            s @ (GIState::Locked | GIState::LockedByMovement) => {
                next_gi_state.set(s);

                if let Ok(player_camera) = player_camera {
                    commands.entity(player_camera).remove::<RaycastPickCamera>();
                }
            }
            // Enable raycasting
            s @ GIState::Unlocked => {
                next_gi_state.set(s);

                if raycast_camera.is_err() {
                    if let Ok(player_camera) = player_camera {
                        commands.entity(player_camera).insert(RaycastPickCamera::default());
                    }
                }
            }
        }
    }
}

pub fn update_tile_state_stable(
    mut tiles: Query<(&Handle<StandardMaterial>, &mut Tile)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gi_lock_sender: EventWriter<GlobalInteractionLockEvent>,
    opt_combat_manager: Option<Res<CombatManager>>,
    player: Query<&Player>,
) {
    if opt_combat_manager.is_some() {
        return;
    }
    for (material_handle, mut tile) in &mut tiles {
        let raw_material = materials.get_mut(material_handle).unwrap();

        let mut current_colour = Color::rgba(1.0, 1.0, 1.0, 0.6);
        if tile.is_hovered {
            current_colour = Color::BLUE;
        } else if let Some(ref sub_region_data) = tile.sub_region_id {
            current_colour.set_a(1.0);
            match sub_region_data.subregion_type {
                SubregionType::UnclearedCombat => {
                    // current_colour.set_r();
                    current_colour.set_g(0.5);
                    current_colour.set_b(0.5);
                }
                SubregionType::ClearedCombat => {
                    current_colour.set_r(0.5);
                    // current_colour.set_g(0.5);
                    current_colour.set_b(0.5);
                }
                SubregionType::Other => {}
            }
        }
        if tile.is_obstructed {
            current_colour = Color::GRAY
        }

        raw_material.base_color = current_colour;

        if tile.is_clicked {
            if let Ok(player) = player.get_single() {
                if player.hex_coord == tile.coord {
                    tile.is_clicked = false;
                    // TODO: Pressed your own tile - inventory?
                } else {
                    gi_lock_sender.send(GlobalInteractionLockEvent(GIState::LockedByMovement));
                }
            }
        }
    }
}

/// Calculates the y rotation of the player when transitioning from a tile to an adjacent tile
///
/// ```
/// Neighbours when r is EVEN     Neighbours when r is ODD
///
///       forwards π/2                  forwards π/2
///            ↑                             ↑
///            |                             |
///            |                             |
///          _____                         _____
///         /     \                       /     \
///   _____/  q+1  \_____           _____/  q+1  \_____
///  /     \   r   /     \         /     \   r   /     \
/// /   q   \_____/   q   \       /  q+1  \_____/  q+1  \
/// \  r-1  /     \  r+1  /       \  r-1  /     \  r+1  /
///  \_____/   q   \_____/         \_____/   q   \_____/  ---→ default 0 y rotation
///  /     \   r   /     \         /     \   r   /     \
/// /  q-1  \_____/  q-1  \       /   q   \_____/   q   \
/// \  r-1  /     \  r+1  /       \  r-1  /     \  r+1  /
///  \_____/  q-1  \_____/         \_____/  q-1  \_____/
///        \   r   /                     \   r   /
///         \_____/                       \_____/
/// ```
/// In line 90 of src/animengine.rs it is stipulated that a rotation of π/2 is "forwards", such
/// that r stays constant moving in the "forwards/backwards" direction
pub fn rotation_to(player: HexCoord, adjacent: HexCoord) -> f32 {
    let forwards = PI / 2.0;

    if player.r == adjacent.r {
        if adjacent.q == player.q + 1 {
            return forwards;
        } else if adjacent.q == player.q - 1 {
            return forwards + PI;
        }
    }

    if player.r % 2 == 0 {
        match (adjacent.q, adjacent.r) {
            (q, r) if q == player.q && r == player.r + 1 => forwards + -PI / 3.0,
            (q, r) if q == player.q - 1 && r == player.r + 1 => forwards + 2.0 * -PI / 3.0,
            (q, r) if q == player.q && r == player.r - 1 => forwards + PI / 3.0,
            (q, r) if q == player.q - 1 && r == player.r - 1 => forwards + 2.0 * PI / 3.0,
            _ => unreachable!(),
        }
    } else {
        match (adjacent.q, adjacent.r) {
            (q, r) if q == player.q + 1 && r == player.r + 1 => forwards + -PI / 3.0,
            (q, r) if q == player.q && r == player.r + 1 => forwards + 2.0 * -PI / 3.0,
            (q, r) if q == player.q + 1 && r == player.r - 1 => forwards + PI / 3.0,
            (q, r) if q == player.q && r == player.r - 1 => forwards + 2.0 * PI / 3.0,
            _ => unreachable!(),
        }
    }
}

pub fn move_player_stable(
    mut tiles: Query<(&Handle<StandardMaterial>, &mut Tile)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gi_lock_sender: EventWriter<GlobalInteractionLockEvent>,
    mut player_query: Query<(&mut Transform, &mut Player, &mut RiggedEntity)>,
    // mut camera_query: Query<(&mut Transform, &mut PlayerCameraMarker), Without<Player>>,
    // mut r_entities: Query<&mut RiggedEntity>,
    time: Res<Time>,
    mut map_context: ResMut<MapContext>,
    chests: Query<(Entity, &Chest)>,
    mut chest_open_sender: EventWriter<ChestOpenEvent>,
    mut combat_manager: Option<ResMut<CombatManager>>,
) {
    let (mut p_transform, mut p, mut p_rentity) = player_query.get_single_mut().unwrap();

    // Calculate only once
    if p.path.is_none() {
        let end_tile = tiles.iter_mut().find_map(|(_, mut t)| {
            if t.is_clicked {
                t.is_clicked = false;
                if !t.is_obstructed {
                    Some(t.coord)
                } else {
                    None
                }
            } else {
                None
            }
        });
        if let Some(end_tile) = end_tile {
            let start_tile = p.hex_coord;

            let obstructed_tiles: Vec<HexCoord> = tiles
                .iter()
                .filter_map(|(_, t)| if t.is_obstructed { Some(t.coord) } else { None })
                .collect();

            let path = astar(start_tile, end_tile, &obstructed_tiles);
            // TODO: Clarify the None case for the astar func... I think input has been sanitised by now (assert isn't
            // panicking so that's a good sign)
            assert!(path.is_some());

            if let Some(_) = combat_manager {
                let mut some_path = path.unwrap();
                some_path.remove(0);
                while some_path.len() > p.remaining_speed as usize {
                    some_path.remove(some_path.len() - 1);
                }
                p.path = Some(some_path);
                p.remaining_speed = p.remaining_speed - p.path.as_ref().unwrap().len() as i32;
            } else {
                p.path = path;
            }

            trace!("path len {}", p.path.as_ref().unwrap().len());

            // Highlight the path yellow
            tiles.for_each(|(material_handle, tile)| {
                if p.path.as_ref().unwrap().contains(&tile.coord) {
                    materials.get_mut(material_handle).unwrap().base_color = Color::YELLOW.with_a(0.6);
                }
            });
        } else {
            gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Unlocked))
        }
    }

    // Timer just finished -> Set player transform (calculate by mapping qr to xz) -> Pend animation

    p.move_timer.tick(time.delta());
    if p.move_timer.just_finished() {
        let pos = p.hex_coord;

        let curr_tile = tiles.iter().find(|(_, tile)| tile.coord == pos);

        // If the player walks into a subregion tile, terminate pathfind and change the map
        if let Some((_, tile)) = curr_tile {
            if let Some(ref subregion_data) = tile.sub_region_id {
                trace!("Changing subregion!");
                map_context.change_map(subregion_data.id.clone());
                p.path = Some(vec![]);
                p.reset_move_timer();
                gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Unlocked));
            }

            // If the player walks into a chest, terminate pathfind, reset things and send an open chest event
            for (chest_ent, chest) in &chests {
                if tile.coord == chest.hex_coord {
                    info!("Opening a chest at coord q={} r={}", tile.coord.q, tile.coord.r);
                    p.path = Some(vec![]);
                    p.reset_move_timer();

                    p_transform.translation.x =
                        tile.coord.q as f32 * HORIZONTAL_SPACING + tile.coord.r as f32 % 2.0 * HOR_OFFSET;
                    p_transform.translation.z = tile.coord.r as f32 * VERTICAL_SPACING;

                    tiles.for_each(|(material_handle, tile)| {
                        let mut colour = materials.get_mut(material_handle).unwrap();
                        if colour.base_color == Color::YELLOW.with_a(0.6) {
                            if tile.sub_region_id.is_none() {
                                colour.base_color = Color::rgba(1.0, 1.0, 1.0, 0.6)
                            } else {
                                match tile.sub_region_id.as_ref().unwrap().subregion_type {
                                    SubregionType::Other => colour.base_color = Color::WHITE,
                                    _ => {}
                                }
                            }
                        }
                    });

                    chest_open_sender.send(ChestOpenEvent {
                        chest_ent,
                        chest: chest.clone(),
                    });

                    return;
                }
            }
        }

        if let Some(player_path) = &mut p.path {
            p_transform.translation.x = pos.q as f32 * HORIZONTAL_SPACING + pos.r as f32 % 2.0 * HOR_OFFSET;
            p_transform.translation.z = pos.r as f32 * VERTICAL_SPACING;

            if !player_path.is_empty() {
                let next_tile = player_path.remove(0);

                if p.hex_coord != next_tile {
                    p_transform.rotation = Quat::from_rotation_y(rotation_to(p.hex_coord, next_tile));
                    p_rentity.pend(9);
                }

                p.hex_coord = next_tile;
            } else {
                p.path = None;
                p.reset_move_timer();
                gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Unlocked));
                // if let Some(mut cb_maager) = opt_combat_manager {
                //     cb_maager.turn = Turn::Player(Phase::Action1)
                // }
                tiles.for_each(|(material_handle, tile)| {
                    let mut colour = materials.get_mut(material_handle).unwrap();
                    if colour.base_color == Color::YELLOW.with_a(0.6) {
                        if tile.sub_region_id.is_none() {
                            colour.base_color = Color::rgba(1.0, 1.0, 1.0, 0.6)
                        } else {
                            match tile.sub_region_id.as_ref().unwrap().subregion_type {
                                SubregionType::Other => colour.base_color = Color::WHITE,
                                _ => {}
                            }
                        }
                    }
                })
            }
        }
    }
}

// pub fn update_tile_pos(
//     mut tiles: Query<(&mut Transform, &mut Tile), (With<Tile>, Without<Player>, Without<PlayerAction>)>,
//     mut combat_manager: ResMut<CombatManager>,
//     mut spell_casts_query: Query<(&mut Transform, &PlayerAction), With<PlayerAction>>,
//     mut enemies: Query<&mut Enemy>,
// ) {
//     for (mut tile_transform, mut tile_struct) in &mut tiles {
//         tile_transform.translation.y = 1.0;
//         if tile_struct.is_hovered {
//             tile_transform.translation.y = 1.1;
//             if let Some(player_action) = &combat_manager.player_action.clone() {
//                 for (mut pos, spell) in &mut spell_casts_query {
//                     if spell == player_action {
//                         pos.translation.x = tile_transform.translation.x;
//                         pos.translation.z = tile_transform.translation.z;
//                         if tile_struct.is_clicked {
//                             combat_manager.reset_buttons = true;
//                             tile_struct.is_clicked = false;
//                             match player_action {
//                                 PlayerAction::Movement => {}
//                                 PlayerAction::Action(action) => match action {
//                                     AcitonType::Fireball => {
//                                         for mut enemy in &mut enemies {
//                                             if enemy.hex_coord == tile_struct.coord {
//                                                 enemy.health.hp -= 5.0
//                                             }
//                                         }
//                                     }
//                                     _ => {}
//                                 },
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         if tile_struct.is_clicked {
//             tile_transform.translation.y = 1.3;
//         }
//     }
// }

// pub fn update_player_pos(
//     mut tiles: Query<(&mut Transform, &mut Tile), (With<Tile>, Without<Player>)>,
//     mut player_data: Query<&mut Player, With<Player>>,
//     mut player_transform: Query<&mut Transform, With<Player>>,
//     time: Res<Time>,
//     mut combat_manager: ResMut<CombatManager>,
//     mut map_context: ResMut<MapContext>,
// ) {
//     // TEMP WORKAROUND DEPRECATE LATER
//     let mut data = player_data.get_single_mut();
//     if data.is_err() {
//         return;
//     }
//     let mut data = data.unwrap();

//     let mut clicked_tiles: Vec<HexCoord> = Vec::new();
//     let mut player_qr = data.hex_coord.clone();
//     let mut end_tile: HexCoord = data.hex_coord.clone();
//     for (_, tile_struct) in &mut tiles {
//         if tile_struct.is_clicked {
//             end_tile = tile_struct.coord;
//             clicked_tiles.push(tile_struct.coord)
//         }
//     }

//     for (_, tile) in &tiles {
//         if tile.coord == data.hex_coord {
//             if !combat_manager.in_combat {
//                 if let Some(subregion_data) = tile.sub_region_id.clone() {
//                     map_context.change_map(subregion_data.id);
//                     combat_manager.reset_buttons = true;
//                     data.path = Some(Vec::new());
//                 }
//             }
//         }
//     }

//     data.move_timer.tick(time.delta());
//     if data.move_timer.just_finished() {
//         let mut p_pos = player_transform.single_mut();
//         if let Some(player_path) = &mut data.path {
//             if !player_path.is_empty() {
//                 p_pos.translation.x =
//                     player_path[0].q as f32 * HORIZONTAL_SPACING + player_path[0].r as f32 % 2.0 * HOR_OFFSET;
//                 p_pos.translation.z = player_path[0].r as f32 * VERTICAL_SPACING;
//                 player_qr = player_path[0];
//                 player_path.remove(0);
//             } else {
//                 // pathfinding done
//                 for tile_qr in clicked_tiles {
//                     for (_, mut tile_struct) in &mut tiles {
//                         // Unlocks tiles and resets button
//                         combat_manager.reset_buttons = true;
//                         if !tile_struct.is_obstructed {
//                             tile_struct.can_be_clicked = true;
//                         }
//                         if tile_struct.coord == tile_qr {
//                             tile_struct.is_clicked = false;
//                         }
//                     }
//                 }
//                 data.path = None
//             }
//         } else if combat_manager.player_action == Some(PlayerAction::Movement) && clicked_tiles.len() == 1 {
//             // TODO: remove the need for samsple_obstructed
//             let sample_obstructed = vec![HexCoord::new(10, 10)];
//             if end_tile != player_qr {
//                 let path = astar(player_qr, end_tile, &sample_obstructed);
//                 if let Some(some_path) = path {
//                     data.path = Some(some_path);
//                     // Lock clicking on tiles while moving
//                     for (_, mut tile_struct) in &mut tiles {
//                         tile_struct.can_be_clicked = false
//                     }
//                 }
//             }
//         }
//     }
//     if player_qr != data.hex_coord {
//         data.hex_coord = player_qr;
//     }
// }
