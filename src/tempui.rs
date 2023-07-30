use crate::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(draw_menu_ui.in_schedule(OnEnter(GameState::Menu)))
            .add_system(play_button_interaction.in_set(OnUpdate(GameState::Menu)))
            .add_system(cleanup_menu.in_schedule(OnExit(GameState::Menu)));
    }
}

#[derive(Component)]
struct MenuCameraMarker;

#[derive(Component)]
struct MenuUIRootMarker;

#[derive(Component)]
struct PlayButtonMarker;

fn draw_menu_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), Name::new("menu_camera"), MenuCameraMarker));

    commands
        // Container
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgb(0.1, 0.1, 0.1).into(),
            ..default()
        })
        .insert(Name::new("menu ui root"))
        .insert(MenuUIRootMarker)
        .with_children(|parent| {
            parent
                // Button
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(20.0), Val::Percent(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgb(0.0, 0.6, 0.1).into(),
                    // Always set FocusPolicy::Block on buttons otherwise you get occasional displacement bugs
                    focus_policy: bevy::ui::FocusPolicy::Block,
                    ..default()
                })
                .insert(Name::new("play button"))
                .insert(PlayButtonMarker)
                .with_children(|parent| {
                    parent
                        // Text
                        .spawn(TextBundle {
                            text: Text::from_section(
                                "Play!",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 40.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });
                });
        });
}

#[allow(clippy::type_complexity)]
fn play_button_interaction(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<PlayButtonMarker>)>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut background_color) in &mut interaction_query {
        match interaction {
            Interaction::Clicked => {
                next_game_state.set(GameState::InGame);
            }
            Interaction::Hovered => *background_color = Color::rgb(0.0, 1.0, 0.0).into(),
            Interaction::None => *background_color = Color::rgb(0.0, 0.8, 0.2).into(),
        }
    }
}

/// Despawns all entities spawned as part of the menu screen. Runs on the exit of [`GameState::Menu`]
fn cleanup_menu(
    mut commands: Commands,
    camera_query: Query<Entity, With<MenuCameraMarker>>,
    uiroot_query: Query<Entity, With<MenuUIRootMarker>>,
) {
    commands.entity(camera_query.get_single().unwrap()).despawn_recursive();
    commands.entity(uiroot_query.get_single().unwrap()).despawn_recursive();
}

pub const NORMAL_BUTTON: Color = Color::rgba(0.15, 0.15, 0.15, 0.85);
pub const HOVERED_BUTTON: Color = Color::rgba(0.25, 0.25, 0.25, 0.95);
pub const PRESSED_BUTTON: Color = Color::rgba(0.35, 0.75, 0.35, 1.0);

#[derive(Component, Clone, Copy)]
pub enum ButtonType {
    Movement,
    Spell(SpellType),
}
#[derive(Clone, Copy, PartialEq, FromReflect, Reflect)]
pub enum SpellType {
    Fireball,
    Placeholder1,
    Placeholder2,
}

#[derive(Component)]
pub struct ToggleButton {
    pub is_on: bool,
}

impl ToggleButton {
    pub fn new() -> Self {
        Self { is_on: false }
    }
}

#[derive(Component)]
pub struct ButtonText {
    pub passive_text: String,
    pub active_text: String,
}

#[derive(Component)]
pub struct DebugText;

pub fn button_reset_system(
    mut interaction_query: Query<(&mut BackgroundColor, &Children, &mut ToggleButton), With<Button>>,
    mut combat_manager: ResMut<CombatManager>,
    mut text_query: Query<&mut Text, Without<DebugText>>,
    other_text_query: Query<&ButtonText>,
    mut debug_text_queery: Query<&mut Text, With<DebugText>>,
    mut tile_queery: Query<&mut Tile>,
) {
    let mut debug_text = debug_text_queery.single_mut();
    if combat_manager.reset_buttons {
        // debug_text.sections[0].value = "resetting buttons".to_string();
        combat_manager.reset_buttons = false;
        if combat_manager.in_combat && combat_manager.player_action != Some(PlayerAction::Movement) {
            combat_manager.turn = Turn::Enemies;
        }
        combat_manager.player_action = None;

        for (mut color, children, mut toggle_state) in &mut interaction_query {
            let mut text = text_query.get_mut(children[0]).unwrap();
            let other_text = other_text_query.get(children[0]).unwrap();
            toggle_state.is_on = false;
            text.sections[0].value = other_text.passive_text.clone();
            *color = NORMAL_BUTTON.into();
        }
    }
}

pub fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &Children,
            &ButtonType,
            &mut ToggleButton,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut combat_manager: ResMut<CombatManager>,
    mut text_query: Query<&mut Text>,
    other_text_queery: Query<&ButtonText>,
    mut tile_queery: Query<&mut Tile>,
) {
    if combat_manager.turn == Turn::Player {
        for (interaction, mut color, children, button_type, mut toggle_state) in &mut interaction_query {
            let mut text = text_query.get_mut(children[0]).unwrap();
            let other_text = other_text_queery.get(children[0]).unwrap();
            if combat_manager.turn == Turn::Player {
                match *interaction {
                    Interaction::Clicked => {
                        toggle_state.is_on = !toggle_state.is_on;
                        if toggle_state.is_on {
                            for mut tile in &mut tile_queery {
                                if !tile.is_obstructed {
                                    tile.can_be_clicked = true
                                }
                            }
                            *color = PRESSED_BUTTON.into();
                            match *button_type {
                                ButtonType::Movement => combat_manager.player_action = Some(PlayerAction::Movement),
                                ButtonType::Spell(spell_type) => match spell_type {
                                    SpellType::Fireball => {
                                        combat_manager.player_action =
                                            Some(PlayerAction::SpellCast(SpellType::Fireball))
                                    }
                                    _ => {}
                                },
                            }
                            text.sections[0].value = other_text.active_text.clone();
                        } else {
                            for mut tile in &mut tile_queery {
                                tile.can_be_clicked = false
                            }
                            *color = HOVERED_BUTTON.into();
                            combat_manager.player_action = None;
                            text.sections[0].value = other_text.passive_text.clone();
                        }
                    }
                    Interaction::Hovered => {
                        if !toggle_state.is_on {
                            *color = HOVERED_BUTTON.into();
                        }
                    }
                    Interaction::None => {
                        if !toggle_state.is_on {
                            *color = NORMAL_BUTTON.into();
                        }
                    }
                }
            }
        }
    }
}

pub fn update_health_bar(
    health: Res<Health>,
    mut health_bar_query: Query<&mut Style, With<HealthBar>>,
    // mut debug_text_query: Query<&mut Text, With<DebugText>>,
) {
    // let mut text = debug_text_query.single_mut();
    // text.sections[0].value = format!("{}", health.hp);
    // Update the components of the collected entities
    for mut style in &mut health_bar_query {
        style.size = Size::new(Val::Percent((health.hp / health.max_hp) * 100.0), Val::Percent(100.0));
    }
}
