use crate::*;

pub const NORMAL_BUTTON: Color = Color::rgba(0.15, 0.15, 0.15, 0.85);
pub const HOVERED_BUTTON: Color = Color::rgba(0.25, 0.25, 0.25, 0.95);
pub const PRESSED_BUTTON: Color = Color::rgba(0.35, 0.75, 0.35, 1.0);

#[derive(Component, Clone, Copy)]
pub enum ButtonType {
    Movement,
    Spell(SpellType),
}
#[derive(Clone, Copy, PartialEq)]
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
    mut combat_manager_query: Query<&mut CombatManager>,
    mut text_query: Query<&mut Text, Without<DebugText>>,
    other_text_query: Query<&ButtonText>,
    mut debug_text_queery: Query<&mut Text, With<DebugText>>,
    mut tile_queery: Query<&mut Tile>,
) {
    let mut debug_text = debug_text_queery.single_mut();
    let mut combat_manager = combat_manager_query.single_mut();
    if combat_manager.reset_buttons {
        for mut tile in &mut tile_queery {
            tile.can_be_clicked = false
        }
        debug_text.sections[0].value = "resetting buttons".to_string();
        combat_manager.reset_buttons = false;
        if combat_manager.player_action != Some(PlayerAction::Movement) {
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
    mut combat_manager_queery: Query<&mut CombatManager>,
    mut text_query: Query<&mut Text>,
    other_text_queery: Query<&ButtonText>,
    mut tile_queery: Query<&mut Tile>,
) {
    let mut combat_manager = combat_manager_queery.single_mut();
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
