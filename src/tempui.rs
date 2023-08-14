use crate::*;
use std::fs;
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
enum MainMenuButton {
    New,
    Load,
}

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
                .insert(Name::new("new game button"))
                .insert(MainMenuButton::New)
                .with_children(|parent| {
                    parent
                        // Text
                        .spawn(TextBundle {
                            text: Text::from_section(
                                "New Game",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 40.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });
                });
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
                .insert(Name::new("load save button"))
                .insert(MainMenuButton::Load)
                .with_children(|parent| {
                    parent
                        // Text
                        .spawn(TextBundle {
                            text: Text::from_section(
                                "Load Save",
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
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &MainMenuButton), Changed<Interaction>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut background_color, button_type) in &mut interaction_query {
        match interaction {
            Interaction::Clicked => match button_type {
                MainMenuButton::New => {
                    let default_world =
                        fs::read_to_string("default_world.json").expect("Something went wrong reading the file");
                    fs::write("world.json", default_world).expect("Unable to write to file");
                    debug!("Reset the World");
                    let default_player =
                        fs::read_to_string("default_player_data.json").expect("Something went wrong reading the file");
                    fs::write("player_data.json", default_player).expect("Unable to write to file");

                    let default_inventory =
                        fs::read_to_string("default_inventory.json").expect("Something went wrong reading the file");
                    fs::write("inventory.json", default_inventory).expect("Unable to write to file");
                    next_game_state.set(GameState::CharacterCreation);
                }
                MainMenuButton::Load => {
                    next_game_state.set(GameState::VisibleLoading);
                }
            },
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

#[derive(Component, Clone, Copy, Debug)]
pub enum ButtonType {
    CombatButton(CombatButtonType),
    NonCombatButton(NonCombatButtonType),
}
#[derive(Clone, Copy, Debug)]
pub enum CombatButtonType {
    Movement,
    Action(AcitonType),
}
#[derive(Clone, Copy, Debug)]
pub enum NonCombatButtonType {
    A,
    B,
}
#[derive(Clone, Copy, PartialEq, FromReflect, Reflect, Debug)]
pub enum AcitonType {
    EndPhase,
    Fireball,
    Smack,
    RunSmack,
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

// pub fn button_reset_system(
//     mut interaction_query: Query<(&mut BackgroundColor, &Children, &mut ToggleButton), With<Button>>,
//     mut combat_manager: ResMut<CombatManager>,
//     other_text_query: Query<&ButtonText>,
//     mut tile_queery: Query<&mut Tile>,
// ) {
//     if combat_manager.reset_buttons {
//         combat_manager.reset_buttons = false;
//         if combat_manager.in_combat && combat_manager.player_action != Some(PlayerAction::Movement) {
//             combat_manager.turn = Turn::Enemies;
//         }
//         combat_manager.player_action = None;

//         for (mut color, children, mut toggle_state) in &mut interaction_query {
//             let other_text = other_text_query.get(children[0]).unwrap();
//             toggle_state.is_on = false;
//             *color = NORMAL_BUTTON.into();
//         }
//     }
// }
