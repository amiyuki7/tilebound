use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_mod_picking::prelude::RaycastPickCamera;
use bevy_scene_hook::{HookedSceneBundle, SceneHook};

pub mod animengine;
pub mod astar;
pub mod load;
pub mod map_load;
pub mod tempui;

pub use animengine::*;
pub use astar::*;
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
    /// TODO: Work with VisibleLoading state after LoadingPhaseTwo
    /// Spawning and parent/child hierarchy setup for entities and cameras
    VisibleLoading,
    /// Player controls ready
    InGame,
}

#[derive(Component, Reflect)]
pub struct Player {
    pub hex_coord: HexCoord,
    pub path: Option<Vec<HexCoord>>,
    pub move_timer: Timer,
}

impl Player {
    /// `move_timer_duration`: The duration of the running animation of the REntity
    /// Access like so:
    /// ```
    /// fn my_system(re_map: Res<REntityMap>) {
    ///     let run_dur = re_map.0.get(&REntityType::Kraug).unwrap().animations[9].duration;
    /// }
    /// ```
    pub fn new(q: i32, r: i32, move_timer_duration: f32) -> Player {
        // Set it close to the just finished amount so we can snap straight into animations during movement
        let mut timer = Timer::from_seconds(move_timer_duration, TimerMode::Repeating);
        timer.set_elapsed(::std::time::Duration::from_secs_f32(move_timer_duration - 0.05));

        Player {
            hex_coord: HexCoord::new(q, r),
            path: None,
            // move_timer: Timer::from_seconds(move_timer_duration, TimerMode::Repeating),
            move_timer: timer,
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

#[derive(Component, Serialize, Deserialize, Reflect, FromReflect, Clone, Debug)]
pub struct Enemy {
    pub hex_coord: HexCoord,
    pub path: Option<Vec<HexCoord>>,
    pub attack_range: i32,
    pub movement_range: i32,
    pub damage: f32,
    pub health: Health,
    #[serde(default, skip)]
    pub move_timer: Timer,
}

impl Enemy {
    pub fn new(q: i32, r: i32, attack_range: i32, movement_range: i32, damage: f32, hp: f32) -> Enemy {
        Enemy {
            hex_coord: HexCoord::new(q, r),
            path: None,
            move_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            attack_range,
            movement_range,
            damage,
            health: Health::new(hp),
        }
    }
}

#[derive(PartialEq, Component, Clone, Reflect, FromReflect)]
pub enum PlayerAction {
    Movement,
    SpellCast(SpellType),
}

#[derive(Resource, PartialEq, Reflect)]
pub struct CombatManager {
    pub in_combat: bool,
    pub turn: Turn,
    pub player_action: Option<PlayerAction>,
    pub reset_buttons: bool,
}

impl CombatManager {
    pub fn new() -> CombatManager {
        CombatManager {
            in_combat: false,
            turn: Turn::Player,
            player_action: None,
            reset_buttons: false,
        }
    }
}
#[derive(PartialEq, Reflect)]
pub enum Turn {
    Player,
    Allies,
    Enemies,
}

#[derive(Component)]
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

pub fn update_tile_pos(
    mut tiles: Query<(&mut Transform, &mut Tile), (With<Tile>, Without<Player>, Without<PlayerAction>)>,
    mut combat_manager: ResMut<CombatManager>,
    mut spell_casts_query: Query<(&mut Transform, &PlayerAction), With<PlayerAction>>,
    mut enemies: Query<&mut Enemy>,
) {
    for (mut tile_transform, mut tile_struct) in &mut tiles {
        tile_transform.translation.y = 1.0;
        if tile_struct.is_hovered {
            tile_transform.translation.y = 1.1;
            if let Some(player_action) = &combat_manager.player_action.clone() {
                for (mut pos, spell) in &mut spell_casts_query {
                    if spell == player_action {
                        pos.translation.x = tile_transform.translation.x;
                        pos.translation.z = tile_transform.translation.z;
                        if tile_struct.is_clicked {
                            combat_manager.reset_buttons = true;
                            tile_struct.is_clicked = false;
                            match player_action {
                                PlayerAction::Movement => {}
                                PlayerAction::SpellCast(spell) => match spell {
                                    SpellType::Fireball => {
                                        for mut enemy in &mut enemies {
                                            if enemy.hex_coord == tile_struct.coord {
                                                enemy.health.hp -= 5.0
                                            }
                                        }
                                    }
                                    _ => {}
                                },
                            }
                        }
                    }
                }
            }
        }
        if tile_struct.is_clicked {
            tile_transform.translation.y = 1.3;
        }
    }
}

pub fn update_player_pos(
    mut tiles: Query<(&mut Transform, &mut Tile), (With<Tile>, Without<Player>)>,
    mut player_data: Query<&mut Player, With<Player>>,
    mut player_transform: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    mut combat_manager: ResMut<CombatManager>,
    mut debug_text_query: Query<&mut Text, With<DebugText>>,
    mut map_context: ResMut<MapContext>,
) {
    let mut debug_text = debug_text_query.single_mut();

    // TEMP WORKAROUND DEPRECATE LATER
    let mut data = player_data.get_single_mut();
    if data.is_err() {
        return;
    }
    let mut data = data.unwrap();

    let mut clicked_tiles: Vec<HexCoord> = Vec::new();
    let mut player_qr = data.hex_coord.clone();
    let mut end_tile: HexCoord = data.hex_coord.clone();
    for (_, tile_struct) in &mut tiles {
        if tile_struct.is_clicked {
            end_tile = tile_struct.coord;
            clicked_tiles.push(tile_struct.coord)
        }
    }

    for (_, tile) in &tiles {
        if tile.coord == data.hex_coord {
            if !combat_manager.in_combat {
                if let Some(subregion_data) = tile.sub_region_id.clone() {
                    map_context.change_map(subregion_data.id);
                    combat_manager.reset_buttons = true;
                    data.path = Some(Vec::new());
                }
            }
        }
    }

    data.move_timer.tick(time.delta());
    if data.move_timer.just_finished() {
        let mut p_pos = player_transform.single_mut();
        if let Some(player_path) = &mut data.path {
            if !player_path.is_empty() {
                p_pos.translation.x =
                    player_path[0].q as f32 * HORIZONTAL_SPACING + player_path[0].r as f32 % 2.0 * HOR_OFFSET;
                p_pos.translation.z = player_path[0].r as f32 * VERTICAL_SPACING;
                player_qr = player_path[0];
                player_path.remove(0);
            } else {
                // pathfinding done
                for tile_qr in clicked_tiles {
                    for (_, mut tile_struct) in &mut tiles {
                        // Unlocks tiles and resets button
                        combat_manager.reset_buttons = true;
                        // debug_text.sections[0].value = "reset_buttons is true".to_string();
                        if !tile_struct.is_obstructed {
                            tile_struct.can_be_clicked = true;
                        }
                        if tile_struct.coord == tile_qr {
                            tile_struct.is_clicked = false;
                        }
                    }
                }
                data.path = None
            }
        } else if combat_manager.player_action == Some(PlayerAction::Movement) && clicked_tiles.len() == 1 {
            // TODO: remove the need for samsple_obstructed
            let sample_obstructed = vec![HexCoord::new(10, 10)];
            if end_tile != player_qr {
                let path = astar(player_qr, end_tile, &sample_obstructed);
                if let Some(some_path) = path {
                    data.path = Some(some_path);
                    // Lock clicking on tiles while moving
                    for (_, mut tile_struct) in &mut tiles {
                        tile_struct.can_be_clicked = false
                    }
                }
            }
        }
    }
    if player_qr != data.hex_coord {
        data.hex_coord = player_qr;
    }
}

pub fn enemy_ai(
    // mut tiles: Query<(&mut Transform, &mut Tile), With<Tile>>,
    mut enemies: Query<(&mut Transform, &mut Enemy)>,
    player_query: Query<&Player>,
    mut combat_manager: ResMut<CombatManager>,
    time: Res<Time>,
    mut player_health: ResMut<Health>,
    mut debug_text_query: Query<&mut Text, With<DebugText>>,
    mut map_context: ResMut<MapContext>,
) {
    let mut debug_text = debug_text_query.single_mut();

    // TEMP WORKAROUND DEPRECATE LATER
    let player = player_query.get_single();
    if player.is_err() {
        return;
    }
    let player = player.unwrap();

    if combat_manager.turn == Turn::Enemies {
        // combat_manager.turn = Turn::Player;
        if !enemies.is_empty() {
            for (mut enemy_pos, mut enemy_data) in &mut enemies {
                if enemy_data.path.is_none() {
                    let sample_obstructed = vec![HexCoord::new(10, 10)];
                    let e_path = astar(enemy_data.hex_coord, player.hex_coord, &sample_obstructed);
                    if let Some(e_some_path) = &mut e_path.clone() {
                        e_some_path.remove(0);
                        while e_some_path.len() > enemy_data.movement_range as usize {
                            e_some_path.remove(e_some_path.len() - 1);
                            // debug_text.sections[0].value = format!("{:#?}", e_some_path.len()).to_string();
                        }
                        // println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n{:#?} \n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n", e_some_path);
                        // e_some_path = *e_some_path[1..enemy_data.movement_range as usize].to_vec();
                        enemy_data.path = Some(e_some_path.clone());
                    }
                }
                if let Some(e_some_path) = &mut enemy_data.path.clone() {
                    enemy_data.move_timer.tick(time.delta());
                    if enemy_data.move_timer.just_finished() {
                        if hex_distance(&enemy_data.hex_coord, &player.hex_coord) > enemy_data.attack_range {
                            enemy_data.hex_coord = e_some_path[0];
                            // debug_text.sections[0].value =
                            // format!("{:#?} {:#?}", e_some_path[0].q, e_some_path[0].r).to_string();
                            if !e_some_path.is_empty() {
                                enemy_pos.translation.x = e_some_path[0].q as f32 * HORIZONTAL_SPACING
                                    + e_some_path[0].r as f32 % 2.0 * HOR_OFFSET;
                                enemy_pos.translation.z = e_some_path[0].r as f32 * VERTICAL_SPACING;
                            };
                            e_some_path.remove(0);
                            enemy_data.path = Some(e_some_path.clone());
                        }
                        // debug_text.sections[0].value =
                        //     format!("{}", hex_distance(&enemy_data.hex_coord, &player.hex_coord).to_string());
                        if hex_distance(&enemy_data.hex_coord, &player.hex_coord) <= enemy_data.attack_range {
                            player_health.hp -= enemy_data.damage;
                            combat_manager.turn = Turn::Player;
                            enemy_data.path = None;
                        }
                    }
                    if e_some_path.len() == 0 {
                        combat_manager.turn = Turn::Player;
                        enemy_data.path = None;
                    }
                }
            }
        } else {
            combat_manager.in_combat = false;
            map_context.clear_combat_data();
            combat_manager.turn = Turn::Player
        }
    }
}

pub fn update_enemy_health(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Enemy)>,
) {
    for (entity, enemy) in query.iter() {
        if enemy.health.hp <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let health_percentage = enemy.health.hp / enemy.health.max_hp;
        let new_material: Handle<StandardMaterial> =
            materials.add(Color::rgba(1.0, 0.0, 0.0, health_percentage).into());

        // Replace the old material with the new one
        commands.entity(entity).insert(new_material);
    }
}
