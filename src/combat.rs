use crate::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(add_combat_stuff.run_if(resource_added::<CombatManager>()))
            .add_systems(
                (
                    combat_system,
                    combat_button_system.in_set(OnUpdate(UIState::Null)),
                    enemy_ai,
                    update_enemy_health,
                    update_player_health,
                    button_reset_system,
                )
                    .distributive_run_if(resource_exists::<CombatManager>()),
            )
            .add_system(remove_combat_stuff.run_if(resource_removed::<CombatManager>()));
    }
}

#[derive(Component, Serialize, Deserialize, Reflect, FromReflect, Clone, Debug)]
pub struct Enemy {
    pub hex_coord: HexCoord,
    #[serde(default, skip_serializing)]
    pub path: Option<Vec<HexCoord>>,
    pub attack_range: i32,
    pub movement_range: i32,
    pub damage: f32,
    pub health: Health,
    #[serde(default, skip)]
    pub ended_turn: bool,
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
            ended_turn: false,
        }
    }
}
#[derive(Reflect, FromReflect, Serialize, Deserialize)]
pub struct RespawnPoint {
    pub world: String,
    pub coord: HexCoord,
}

#[derive(Resource, PartialEq, Reflect, Debug)]
pub struct CombatManager {
    pub turn: Turn,
    pub player_action: Option<AcitonType>,
    pub reset_buttons: bool,
}

impl CombatManager {
    pub fn new() -> CombatManager {
        CombatManager {
            turn: Turn::Player(Phase::Movement),
            player_action: None,
            reset_buttons: false,
        }
    }
}

#[derive(PartialEq, Reflect, FromReflect, Clone, Debug)]
pub enum Turn {
    Player(Phase),
    // Allies,
    Enemies,
}

#[derive(PartialEq, Reflect, FromReflect, Clone, Debug)]
pub enum Phase {
    Movement,
    Action1,
    Action2,
}

pub fn combat_system(
    mut combat_manager: ResMut<CombatManager>,
    mut tiles: Query<(&Handle<StandardMaterial>, &mut Tile)>,
    mut spells: Query<(&mut Transform, &Spell)>,
    mut enemies: Query<&mut Enemy>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gi_lock_sender: EventWriter<GlobalInteractionLockEvent>,
    player_query: Query<&Player>,
) {
    // Response to player chosing action
    let player = player_query.single();
    'outer: for (tile_mat, tile) in &mut tiles {
        let mut raw_mat = materials.get_mut(tile_mat).unwrap();
        if player.path.is_none() {
            if !tile.is_obstructed {
                raw_mat.base_color = Color::rgba(1.0, 1.0, 1.0, 0.6);
            }
            if tile.is_hovered {
                for enemy in &enemies {
                    if enemy.hex_coord == tile.coord {
                        continue 'outer;
                    }
                }
                raw_mat.base_color = Color::BLUE;
            }
        }
    }
    if let Some(player_action) = combat_manager.player_action {
        match player_action {
            AcitonType::Fireball => {
                for (tile_mat, tile) in &mut tiles {
                    let mut raw_mat = materials.get_mut(tile_mat).unwrap();
                    if !tile.is_obstructed {
                        raw_mat.base_color = Color::LIME_GREEN;
                    }
                }
            }
            AcitonType::Smack => {
                let adjactent = get_neighbors(&player.hex_coord);
                for (tile_mat, tile) in &mut tiles {
                    let mut raw_mat = materials.get_mut(tile_mat).unwrap();
                    if !tile.is_obstructed && adjactent.contains(&tile.coord) {
                        raw_mat.base_color = Color::LIME_GREEN;
                    }
                }
            }
            AcitonType::RunSmack => {
                let possible_tiles = vec![
                    HexCoord::new(player.hex_coord.q + 2, player.hex_coord.r),
                    HexCoord::new(player.hex_coord.q - 2, player.hex_coord.r),
                    HexCoord::new(player.hex_coord.q + 1, player.hex_coord.r + 2),
                    HexCoord::new(player.hex_coord.q + 1, player.hex_coord.r - 2),
                    HexCoord::new(player.hex_coord.q - 1, player.hex_coord.r + 2),
                    HexCoord::new(player.hex_coord.q - 1, player.hex_coord.r - 2),
                ];
                for (tile_mat, tile) in &mut tiles {
                    let mut raw_mat = materials.get_mut(tile_mat).unwrap();
                    if !tile.is_obstructed && possible_tiles.contains(&tile.coord) {
                        raw_mat.base_color = Color::LIME_GREEN;
                    }
                }
            }
            _ => {}
        }
    } else {
    }
    // Execute action
    'outer: for (_, mut tile) in &mut tiles {
        if tile.is_clicked {
            if let Some(player_action) = combat_manager.player_action {
                tile.is_clicked = false;
                let mut action_complete = false;
                match player_action {
                    AcitonType::Fireball => {
                        action_complete = true;
                        for (mut pos, spell) in &mut spells {
                            if *spell == Spell::Fireball {
                                pos.translation.x =
                                    tile.coord.q as f32 * HORIZONTAL_SPACING + tile.coord.r as f32 % 2.0 * HOR_OFFSET;
                                pos.translation.z = tile.coord.r as f32 * VERTICAL_SPACING;
                                for mut enemy in &mut enemies {
                                    if enemy.hex_coord == tile.coord {
                                        enemy.health.hp -= 10.0 * player.stats.damage as f32
                                    }
                                }
                            }
                        }
                    }
                    AcitonType::Smack => {
                        if !get_neighbors(&player.hex_coord).contains(&tile.coord) {
                            continue 'outer;
                        }
                        action_complete = true;
                        for mut enemy in &mut enemies {
                            if enemy.hex_coord == tile.coord {
                                enemy.health.hp -= (player.stats.damage * 2) as f32
                            }
                        }
                    }
                    AcitonType::RunSmack => {
                        if !vec![
                            HexCoord::new(player.hex_coord.q + 2, player.hex_coord.r),
                            HexCoord::new(player.hex_coord.q - 2, player.hex_coord.r),
                            HexCoord::new(player.hex_coord.q + 1, player.hex_coord.r + 2),
                            HexCoord::new(player.hex_coord.q + 1, player.hex_coord.r - 2),
                            HexCoord::new(player.hex_coord.q - 1, player.hex_coord.r + 2),
                            HexCoord::new(player.hex_coord.q - 1, player.hex_coord.r - 2),
                        ]
                        .contains(&tile.coord)
                        {
                            continue 'outer;
                        }
                        action_complete = true;
                        for mut enemy in &mut enemies {
                            if enemy.hex_coord == tile.coord {
                                enemy.health.hp -= (player.stats.damage * 5) as f32
                            }
                        }
                    }
                    _ => {}
                }
                if action_complete {
                    combat_manager.reset_buttons = true;
                    combat_manager.player_action = None;
                    if combat_manager.turn == Turn::Player(Phase::Action1) {
                        combat_manager.turn = Turn::Player(Phase::Action2)
                    } else if combat_manager.turn == Turn::Player(Phase::Action2) {
                        combat_manager.turn = Turn::Enemies;
                        for mut enemy in &mut enemies {
                            enemy.ended_turn = false;
                        }
                    }
                }
            }
            if combat_manager.turn == Turn::Player(Phase::Movement) {
                for enemy in &enemies {
                    if enemy.hex_coord == tile.coord {
                        tile.is_clicked = false;
                        continue 'outer;
                    }
                }
                gi_lock_sender.send(GlobalInteractionLockEvent(GIState::LockedByMovement));
            }
        }
    }
}

pub fn button_reset_system(
    mut combat_manager: ResMut<CombatManager>,
    mut buttons: Query<(&mut BackgroundColor, Option<&mut ToggleButton>), With<ToggleButton>>,
) {
    if combat_manager.reset_buttons == true {
        combat_manager.reset_buttons = false;
        for (mut bg_colour, opt_toggle) in &mut buttons {
            *bg_colour = NORMAL_BUTTON.into();
            if let Some(mut toggle) = opt_toggle {
                toggle.is_on = false;
            }
        }
    }
}

pub fn combat_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &Children,
            &ButtonType,
            Option<&mut ToggleButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut combat_manager: ResMut<CombatManager>,
    mut gi_lock_sender: EventWriter<GlobalInteractionLockEvent>,
    mut enemies: Query<&mut Enemy>,
    // mut text_query: Query<&mut Text>,
    // other_text_queery: Query<&ButtonText>,
    // mut tile_queery: Query<&mut Tile>,
) {
    for (interaction, mut color, _children, button_type, mut toggle_state) in &mut interaction_query {
        // let msg = format!("{:#?}", button_type);
        // trace!(msg);
        if let Turn::Player(ref mut phase) = combat_manager.turn.clone() {
            if let ButtonType::CombatButton(cb_type) = *button_type {
                match interaction {
                    Interaction::Clicked => {
                        match cb_type {
                            CombatButtonType::Movement => {}
                            CombatButtonType::Action(action) => match action {
                                AcitonType::EndPhase => {
                                    *color = PRESSED_BUTTON.into();
                                    match phase {
                                        Phase::Movement => {
                                            combat_manager.turn = Turn::Player(Phase::Action1);
                                            gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Locked));
                                            //
                                        }
                                        Phase::Action1 => {
                                            combat_manager.turn = Turn::Player(Phase::Action2);
                                            gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Locked));
                                            //
                                        }
                                        Phase::Action2 => {
                                            combat_manager.turn = Turn::Enemies;
                                            for mut enemy in &mut enemies {
                                                enemy.ended_turn = false;
                                            }
                                            gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Locked));
                                            //
                                        }
                                    }
                                }
                                AcitonType::Fireball => {
                                    if combat_manager.turn == Turn::Player(Phase::Action1)
                                        || combat_manager.turn == Turn::Player(Phase::Action2)
                                    {
                                        if let Some(mut toggle) = toggle_state {
                                            toggle.is_on = !toggle.is_on;
                                            if toggle.is_on {
                                                *color = PRESSED_BUTTON.into();
                                                combat_manager.player_action = Some(AcitonType::Fireball);
                                                gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Unlocked))
                                            } else {
                                                *color = HOVERED_BUTTON.into();
                                                combat_manager.player_action = None;
                                                gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Locked))
                                            }
                                        }
                                    }
                                }
                                AcitonType::Smack => {
                                    if combat_manager.turn == Turn::Player(Phase::Action1)
                                        || combat_manager.turn == Turn::Player(Phase::Action2)
                                    {
                                        if let Some(mut toggle) = toggle_state {
                                            toggle.is_on = !toggle.is_on;
                                            if toggle.is_on {
                                                *color = PRESSED_BUTTON.into();
                                                combat_manager.player_action = Some(AcitonType::Smack);
                                                gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Unlocked));
                                            } else {
                                                *color = HOVERED_BUTTON.into();
                                                combat_manager.player_action = None;
                                                gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Locked))
                                            }
                                        }
                                    }
                                }
                                AcitonType::RunSmack => {
                                    if combat_manager.turn == Turn::Player(Phase::Action1)
                                        || combat_manager.turn == Turn::Player(Phase::Action2)
                                    {
                                        if let Some(mut toggle) = toggle_state {
                                            toggle.is_on = !toggle.is_on;
                                            if toggle.is_on {
                                                *color = PRESSED_BUTTON.into();
                                                combat_manager.player_action = Some(AcitonType::RunSmack);
                                                gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Unlocked));
                                            } else {
                                                *color = HOVERED_BUTTON.into();
                                                combat_manager.player_action = None;
                                                gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Locked))
                                            }
                                        }
                                    }
                                }
                            },
                        }
                    }
                    Interaction::Hovered => {
                        if let Some(toggle) = toggle_state {
                            if !toggle.is_on {
                                *color = HOVERED_BUTTON.into();
                            }
                        } else {
                            *color = HOVERED_BUTTON.into();
                        }
                    }
                    Interaction::None => {
                        if let Some(toggle) = toggle_state {
                            if !toggle.is_on {
                                *color = NORMAL_BUTTON.into();
                            }
                        } else {
                            *color = NORMAL_BUTTON.into();
                        }
                    }
                }
            }
        }
    }
}

pub fn update_player_health(
    mut player_query: Query<(&mut Player, &mut Transform)>,
    mut health_bar_query: Query<&mut Style, With<HealthBar>>,
    mut map_context: ResMut<MapContext>,
    mut commands: Commands,
) {
    let (mut player, mut p_transform) = player_query.single_mut();
    // text.sections[0].value = format!("{}", health.hp);
    // Update the components of the collected entities
    for mut style in &mut health_bar_query {
        style.size = Size::new(
            Val::Percent((player.health.hp / player.health.max_hp) * 100.0),
            Val::Percent(100.0),
        );
    }
    if player.health.hp <= 0.0 {
        commands.remove_resource::<CombatManager>();
        map_context.change_map(player.respawn_point.world.clone());
        player.health.hp = player.health.max_hp;
        player.hex_coord = player.respawn_point.coord;
        p_transform.translation.x =
            player.hex_coord.q as f32 * HORIZONTAL_SPACING + player.hex_coord.r as f32 % 2.0 * HOR_OFFSET;
        p_transform.translation.z = player.hex_coord.r as f32 * VERTICAL_SPACING;
    }
}

pub fn enemy_ai(
    tiles: Query<&Tile, With<Tile>>,
    mut enemies: Query<(&mut Transform, &mut Enemy)>,
    mut player_query: Query<&mut Player>,
    mut combat_manager: ResMut<CombatManager>,
    time: Res<Time>,
    mut gi_lock_sender: EventWriter<GlobalInteractionLockEvent>,
) {
    // TEMP WORKAROUND DEPRECATE LATER
    let player = player_query.get_single_mut();
    if player.is_err() {
        return;
    }
    let mut player = player.unwrap();

    if combat_manager.turn == Turn::Enemies {
        // combat_manager.turn = Turn::Player;
        if !enemies.is_empty() {
            let mut obstructed_tiles: Vec<HexCoord> = tiles
                .iter()
                .filter_map(|t| if t.is_obstructed { Some(t.coord) } else { None })
                .collect();
            obstructed_tiles.push(player.hex_coord);
            for (_, enemy) in &enemies {
                obstructed_tiles.push(enemy.hex_coord);
            }
            for (mut enemy_pos, mut enemy_data) in &mut enemies {
                if enemy_data.ended_turn {
                    continue;
                }
                if enemy_data.path.is_none() {
                    let mut obstructed_without_me = obstructed_tiles.clone();
                    obstructed_without_me.retain(|&x| x != enemy_data.hex_coord);
                    let e_path = astar(enemy_data.hex_coord, player.hex_coord, &obstructed_without_me);
                    if let Some(e_some_path) = &mut e_path.clone() {
                        e_some_path.remove(0);
                        while e_some_path.len() > enemy_data.movement_range as usize {
                            e_some_path.remove(e_some_path.len() - 1);
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
                            if !e_some_path.is_empty() {
                                enemy_pos.translation.x = e_some_path[0].q as f32 * HORIZONTAL_SPACING
                                    + e_some_path[0].r as f32 % 2.0 * HOR_OFFSET;
                                enemy_pos.translation.z = e_some_path[0].r as f32 * VERTICAL_SPACING;
                            };
                            e_some_path.remove(0);
                            enemy_data.path = Some(e_some_path.clone());
                        }
                        if hex_distance(&enemy_data.hex_coord, &player.hex_coord) <= enemy_data.attack_range {
                            player.health.hp -= enemy_data.damage;
                            enemy_data.ended_turn = true;
                            enemy_data.path = None;
                        }
                    }
                    if e_some_path.len() == 0 {
                        enemy_data.ended_turn = true;
                        enemy_data.path = None;
                    }
                }
            }
            let mut ended_turn = true;
            for (_, enemy_data) in &mut enemies {
                if !enemy_data.ended_turn {
                    ended_turn = false
                }
            }
            if ended_turn {
                gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Unlocked));
                combat_manager.turn = Turn::Player(Phase::Movement);
                player.remaining_speed = player.stats.speed;
            }
        }
    }
}

pub fn update_enemy_health(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemies: Query<(Entity, &Enemy)>,
    mut gi_lock_sender: EventWriter<GlobalInteractionLockEvent>,
    mut map_context: ResMut<MapContext>,
) {
    for (entity, enemy) in enemies.iter() {
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
    if enemies.is_empty() {
        gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Unlocked));
        map_context.clear_combat_data();
        commands.remove_resource::<CombatManager>();
    }
}

#[derive(Component)]
pub struct CombatObject;

pub fn remove_combat_stuff(mut commands: Commands, ui_stuff: Query<Entity, With<CombatObject>>) {
    for ui_thing in &ui_stuff {
        commands.entity(ui_thing).despawn_recursive()
    }
}

pub fn add_combat_stuff(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 2.0 * SCALE,
                sectors: 8,
                stacks: 8,
            })),
            material: materials.add(Color::rgb(1.0, 0.7, 0.4).into()),
            transform: Transform::from_xyz(-1000.0, 2.0, -1000.0),
            ..default()
        })
        .insert(Spell::Fireball)
        .insert(CombatObject);

    commands
        // Container
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(15.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position: UiRect::top(Val::Percent(85.0)),
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.1).into(),
            ..default()
        })
        .insert(Name::new("combat ui"))
        .insert(CombatObject)
        .with_children(|parent| {
            let fireball_icon = asset_server.load("2D/Fireball.png");
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::height(Val::Percent(100.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            right: Val::Px(0.0),
                            top: Val::Px(0.0),
                            ..default()
                        },
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    image: UiImage::new(fireball_icon),
                    ..default()
                })
                .insert(ButtonType::CombatButton(CombatButtonType::Action(AcitonType::Fireball)))
                .insert(ToggleButton::new())
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "Fireball",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ))
                        .insert(ButtonText {
                            active_text: "Casting".to_string(),
                            passive_text: "Fireball".to_string(),
                        });
                });
            let smack_icon = asset_server.load("2D/Smack.png");
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::height(Val::Percent(100.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            right: Val::Px(200.0),
                            top: Val::Px(0.0),
                            ..default()
                        },
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    image: UiImage::new(smack_icon),
                    ..default()
                })
                .insert(ButtonType::CombatButton(CombatButtonType::Action(AcitonType::Smack)))
                .insert(ToggleButton::new())
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "Smack",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ))
                        .insert(ButtonText {
                            active_text: "Smacking".to_string(),
                            passive_text: "Smack".to_string(),
                        });
                });
            let run_smack_icon = asset_server.load("2D/RunSmack.png");
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::height(Val::Percent(100.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            right: Val::Px(400.0),
                            top: Val::Px(0.0),
                            ..default()
                        },
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    image: UiImage::new(run_smack_icon),
                    ..default()
                })
                .insert(ButtonType::CombatButton(CombatButtonType::Action(AcitonType::RunSmack)))
                .insert(ToggleButton::new())
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "Run'n'Smack",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ))
                        .insert(ButtonText {
                            active_text: "Running 'n' Smacking".to_string(),
                            passive_text: "Run'n'Smack".to_string(),
                        });
                });
            // parent
            //     .spawn(ButtonBundle {
            //         style: Style {
            //             size: Size::height(Val::Percent(100.0)),
            //             // horizontally center child text
            //             justify_content: JustifyContent::Center,
            //             // vertically center child text
            //             align_items: AlignItems::Center,
            //             position_type: PositionType::Absolute,
            //             position: UiRect {
            //                 right: Val::Px(128.0),
            //                 top: Val::Px(0.0),
            //                 ..default()
            //             },
            //             ..default()
            //         },
            //         background_color: NORMAL_BUTTON.into(),
            //         image: UiImage::new(movement_icon),
            //         ..default()
            //     })
            //     .insert(ButtonType::CombatButton(CombatButtonType::Movement))
            //     .insert(ToggleButton::new())
            //     .with_children(|parent| {
            //         parent
            //             .spawn(TextBundle::from_section(
            //                 "Move",
            //                 TextStyle {
            //                     font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            //                     font_size: 40.0,
            //                     color: Color::rgb(0.9, 0.9, 0.9),
            //                 },
            //             ))
            //             .insert(ButtonText {
            //                 active_text: "Moving".to_string(),
            //                 passive_text: "Move".to_string(),
            //             });
            //     });
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::height(Val::Percent(100.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            ..default()
                        },
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(ButtonType::CombatButton(CombatButtonType::Action(AcitonType::EndPhase)))
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "End Phase",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ))
                        .insert(ButtonText {
                            active_text: "Ended Phase".to_string(),
                            passive_text: "End Phase".to_string(),
                        });
                });
        });
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(200.0), Val::Px(50.0)),
                border: UiRect::all(Val::Px(2.0)),
                position_type: PositionType::Absolute,
                // position: UiRect::new(Val::Px(0.0), Val::Px(200.0), Val::Px(0.0), Val::Px(100.0)),
                ..Default::default()
            },
            background_color: Color::GRAY.into(),
            ..Default::default()
        })
        .insert(CombatObject)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::all(Val::Percent(100.0)),
                        ..Default::default()
                    },
                    background_color: Color::GREEN.into(),
                    ..Default::default()
                })
                .insert(HealthBar);
        });
}
