use bevy_mod_picking::prelude::RaycastPickCamera;

use crate::*;

use std::collections::hash_map::DefaultHasher;
use std::f32::consts::PI;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use bevy_panorbit_camera::*;

pub struct AnimEnginePlugin;

impl Plugin for AnimEnginePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HashMarker>()
            .register_type::<RiggedEntity>()
            .add_event::<SpawnEntityEvent>()
            .add_plugin(PanOrbitCameraPlugin)
            .add_system(spawn_rigged_entity.in_set(OnUpdate(GameState::VisibleLoading)))
            .add_systems((background_animation, key_animation_mock).in_set(OnUpdate(GameState::VisibleLoading)))
            .add_systems(
                (spawn_rigged_entity, background_animation, key_animation_mock).in_set(OnUpdate(GameState::InGame)),
            );
    }
}

#[derive(Reflect, FromReflect, Debug)]
pub(crate) struct CurrentAnimation {
    index: usize,
    duration: f32,
}

#[derive(Reflect, Default, Debug)]
pub(crate) enum IdleState {
    #[default]
    Unarmed,
    Armed,
}

pub struct SpawnEntityEvent {
    pub entity_type: REntityType,
    pub is_player: bool,
}

/// A Rigged Entity
#[derive(Component, Reflect, Debug)]
pub struct RiggedEntity {
    is_player: bool,
    entity_type: REntityType,
    idle_state: IdleState,
    current_animation: Option<CurrentAnimation>,
    pending_animation: Option<usize>,
}

impl RiggedEntity {
    pub fn pend(&mut self, idx: usize) {
        assert!((0..=10).contains(&idx));
        self.pending_animation = Some(idx);
    }
}

/// Used to identify corresponding [`AnimationPlayer`] and [`RiggedEntity`] components
#[derive(Component, Reflect)]
pub struct HashMarker(pub u64);

#[derive(Component)]
pub struct WeaponAnchor;

pub fn spawn_rigged_entity(
    mut commands: Commands,
    mut spawn_entity_event: EventReader<SpawnEntityEvent>,
    re_map: Res<REntityMap>,
) {
    for event in spawn_entity_event.iter() {
        let is_player = event.is_player;
        let mut hasher = DefaultHasher::new();

        let spawned_entity = match event.entity_type {
            REntityType::Kraug => {
                0.hash(&mut hasher);
                let pair_hash = hasher.finish();

                commands
                    .spawn(HookedSceneBundle {
                        scene: SceneBundle {
                            scene: re_map.0.get(&REntityType::Kraug).unwrap().scene.clone_weak(),
                            transform: Transform::from_xyz(0.0, 1.0, 0.0),
                            ..default()
                        },
                        hook: SceneHook::new(move |entity, commands| {
                            if entity.get::<AnimationPlayer>().is_some() {
                                debug!("ANIMATION PLAYER HASH {:?}", pair_hash);
                            }

                            commands.insert(HashMarker(pair_hash));

                            if let Some("mixamorig:RightHandMiddle4") = entity.get::<Name>().map(|t| t.as_str()) {
                                commands.insert(WeaponAnchor);
                                // commands.add_child(axe);
                            }
                        }),
                    })
                    .insert(RiggedEntity {
                        is_player,
                        entity_type: event.entity_type,
                        idle_state: default(),
                        current_animation: None,
                        // 3 is idle_unarmed which will be default
                        pending_animation: Some(3),
                    })
                    .insert(Name::new("Kraug"))
                    .insert(HashMarker(pair_hash))
                    .id()
            }
        };

        if event.is_player {
            // Attach a camera to the player
            let camera = commands
                .spawn((
                    Camera3dBundle {
                        // transform: Transform::from_xyz(0.0, 5.0, -8.0)
                        transform: Transform::from_xyz(0.0, 25.0, -20.0)
                            // Increase x rotation a bit -> more "birds eye"
                            // Decrease x rotation a bit -> more "look at the sky"
                            .with_rotation(
                                /*Quat::from_rotation_x(PI * 13.0 / 12.0)*/
                                Quat::from_rotation_x(4.0) * Quat::from_rotation_z(PI),
                            ),
                        ..default()
                    },
                    PanOrbitCamera {
                        // Disable orbit
                        orbit_sensitivity: 0.0,
                        button_pan: MouseButton::Right,
                        zoom_lower_limit: Some(3.0),
                        zoom_upper_limit: Some(50.0),
                        ..default()
                    },
                    RaycastPickCamera::default(),
                ))
                .insert(PlayerCameraMarker)
                .id();

            commands
                .entity(spawned_entity)
                .insert(Player::new(0, 0))
                .add_child(camera);
        }
    }
}

#[derive(Component)]
pub struct PlayerCameraMarker;

pub fn background_animation(
    re_map: Res<REntityMap>,
    mut r_entities: Query<(&mut RiggedEntity, &HashMarker)>,
    mut anim_players: Query<(&mut AnimationPlayer, &HashMarker)>,
) {
    let hash_to_type: HashMap<u64, REntityType> = r_entities
        .iter()
        .map(|(r_entity, hash)| (hash.0, r_entity.entity_type))
        .collect();

    for (mut anim_player, hash) in &mut anim_players {
        // Safe to unwrap - there will alwauys be a corresponding AnimationPlayer for a RiggedEntity
        let mut rentity = r_entities.iter_mut().find(|&(_, r_hash)| r_hash.0 == hash.0).unwrap().0;

        if let Some(re_type) = hash_to_type.get(&hash.0) {
            if let Some(idx) = rentity.pending_animation.take() {
                rentity.current_animation = Some(CurrentAnimation {
                    index: idx,
                    duration: re_map.0.get(re_type).unwrap().animations[idx].duration,
                });

                // 100% safe to unwrap
                let target_anim =
                    &re_map.0.get(re_type).unwrap().animations[rentity.current_animation.as_ref().unwrap().index];

                anim_player.start_with_transition(target_anim.handle.clone_weak(), Duration::from_secs_f32(0.5));
            }

            // 100% safe to unwrap
            let target_anim =
                &re_map.0.get(re_type).unwrap().animations[rentity.current_animation.as_ref().unwrap().index];

            let anim_elapsed = anim_player.elapsed();
            // trace!("Elapsed {} / {}", anim_elapsed, target_anim.duration);

            // // Linear blend all but index 8 and 9 (move/run)
            // // Move [8] or Run [9] should be "looping" (re-pending itself) unless explicitly overriden
            // if anim_player.elapsed() > target_anim.duration {
            //     // Move animation
            //     if target_anim.handle == re_map.0.get(re_type).unwrap().animations[8].handle {
            //         rentity.pend(8);
            //         return;
            //     }
            //
            //     if target_anim.handle == re_map.0.get(re_type).unwrap().animations[9].handle {
            //         rentity.pend(9);
            //         return;
            //     }
            // }
            //
            if anim_player.elapsed() > target_anim.duration - 0.5
                && target_anim.handle != re_map.0.get(re_type).unwrap().animations[8].handle
                && target_anim.handle != re_map.0.get(re_type).unwrap().animations[9].handle
            {
                match rentity.idle_state {
                    IdleState::Unarmed => rentity.pend(3),
                    IdleState::Armed => rentity.pend(2),
                }
            }

            if anim_player.elapsed() > target_anim.duration
                && (target_anim.handle == re_map.0.get(re_type).unwrap().animations[8].handle
                    || target_anim.handle == re_map.0.get(re_type).unwrap().animations[9].handle)
            {
                match rentity.idle_state {
                    IdleState::Unarmed => rentity.pend(3),
                    IdleState::Armed => rentity.pend(2),
                }
            }
        }
    }
}

pub fn key_animation_mock(keys: Res<Input<KeyCode>>, mut r_entities: Query<&mut RiggedEntity>) {
    for mut rentity in &mut r_entities {
        if rentity.is_player {
            for key in keys.get_just_released() {
                match key {
                    // TODO: Write a proc macro that generates these branches
                    KeyCode::Key1 => {
                        rentity.idle_state = IdleState::Armed;
                        rentity.pend(0);
                    }
                    KeyCode::Key2 => {
                        rentity.idle_state = IdleState::Unarmed;
                        rentity.pend(1);
                    }
                    KeyCode::Key3 => rentity.pend(4),
                    KeyCode::Key4 => rentity.pend(5),
                    KeyCode::Key5 => rentity.pend(6),
                    KeyCode::Key6 => rentity.pend(7),
                    KeyCode::Key7 => rentity.pend(8),
                    KeyCode::Key8 => rentity.pend(9),
                    KeyCode::Key9 => rentity.pend(10),
                    _ => {}
                }
            }
        }
    }
}
