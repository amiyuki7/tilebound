use std::fs;

use crate::*;

pub struct CharacterCreationPlugin;

impl Plugin for CharacterCreationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Stats>()
            .add_system(add_character_creation_ui.in_schedule(OnEnter(GameState::CharacterCreation)))
            .add_system(character_creation_system.in_set(OnUpdate(GameState::CharacterCreation)))
            .add_system(remove_character_creation_ui.in_schedule(OnExit(GameState::CharacterCreation)));
    }
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
pub struct Stats {
    pub speed: i32,
    pub damage: i32,
    pub health: i32,
}

impl Stats {
    pub fn to_tupple(&self) -> (i32, i32, i32) {
        (self.speed, self.damage, self.health)
    }
}

#[derive(Component)]
struct CharacterCreationUI;
#[derive(Component)]
struct ModifyStatButton {
    modification_value: i32,
    stat_type: Stat,
}
#[derive(Component, Debug, Clone, Copy)]
enum Stat {
    Speed,
    Damage,
    Health,
}
#[derive(Component)]
struct EndCharacterCreationButton;

fn add_character_creation_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2dBundle::default(),
        Name::new("character creation camera"),
        CharacterCreationUI,
    ));

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::rgb(0.1, 0.1, 0.1).into(),
            ..default()
        })
        .insert(Stats {
            speed: 1,
            damage: 1,
            health: 1,
        })
        .insert(CharacterCreationUI)
        .with_children(|parent| {
            create_stat_ui(parent, &asset_server, Stat::Speed);
            create_stat_ui(parent, &asset_server, Stat::Damage);
            create_stat_ui(parent, &asset_server, Stat::Health);

            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(300.0), Val::Px(40.0)), // Set width to 100px and height to 100%
                        ..Default::default()
                    },
                    background_color: Color::GREEN.into(),
                    ..Default::default()
                })
                .insert(EndCharacterCreationButton)
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "End Customisation",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::WHITE,
                            },
                        ),
                        ..Default::default()
                    });
                });
        });
}

fn create_stat_ui(parent: &mut ChildBuilder, asset_server: &Res<AssetServer>, stat_type: Stat) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(100.0), Val::Px(40.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "-",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::WHITE,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(ModifyStatButton {
                    modification_value: -1,
                    stat_type,
                });

            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        format!("{:#?}: 0", stat_type),
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..Default::default()
                })
                .insert(stat_type);

            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(100.0), Val::Percent(100.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "+",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::WHITE,
                            },
                        ),
                        ..Default::default()
                    });
                })
                .insert(ModifyStatButton {
                    modification_value: 1,
                    stat_type,
                });
        });
}

fn character_creation_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ModifyStatButton),
        (With<Button>, Changed<Interaction>, Without<EndCharacterCreationButton>),
    >,
    mut end_button_query: Query<(&Interaction, &mut BackgroundColor), With<EndCharacterCreationButton>>,
    mut stats_query: Query<&mut Stats>,
    mut text_query: Query<(&mut Text, &Stat)>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let mut stats = stats_query.single_mut();

    for (interaction, mut background_color, modify_stat_button) in &mut interaction_query {
        match interaction {
            Interaction::Clicked => match modify_stat_button.stat_type {
                Stat::Speed => {
                    if stats.speed + modify_stat_button.modification_value <= 10 - stats.damage - stats.health
                        && stats.speed + modify_stat_button.modification_value >= 1
                    {
                        stats.speed += modify_stat_button.modification_value
                    }
                }
                Stat::Damage => {
                    if stats.damage + modify_stat_button.modification_value <= 10 - stats.speed - stats.health
                        && stats.damage + modify_stat_button.modification_value >= 1
                    {
                        stats.damage += modify_stat_button.modification_value
                    }
                }
                Stat::Health => {
                    if stats.health + modify_stat_button.modification_value <= 10 - stats.damage - stats.speed
                        && stats.health + modify_stat_button.modification_value >= 1
                    {
                        stats.health += modify_stat_button.modification_value
                    }
                }
            },
            Interaction::Hovered => {
                *background_color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *background_color = NORMAL_BUTTON.into();
            }
        }
    }
    for (mut text, stat) in text_query.iter_mut() {
        match stat {
            Stat::Speed => text.sections[0].value = format!("Speed: {}", stats.speed),
            Stat::Damage => text.sections[0].value = format!("Damage: {}", stats.damage),
            Stat::Health => text.sections[0].value = format!("Health: {}", stats.health),
        }
    }

    for (interaction_end, mut background_color) in &mut end_button_query {
        match interaction_end {
            Interaction::Clicked => next_game_state.set(GameState::VisibleLoading),
            Interaction::Hovered => {
                *background_color = HOVERED_BUTTON.into(); // Replace with the desired hover color
            }
            Interaction::None => {
                *background_color = NORMAL_BUTTON.into(); // Replace with the normal color
            }
        }
    }
}

fn remove_character_creation_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<CharacterCreationUI>>,
    stats_query: Query<&mut Stats>,
    mut map_context: ResMut<MapContext>,
) {
    let current_player = fs::read_to_string("player_data.json").expect("Something went wrong reading the file");
    let mut deserialised: Player = serde_json::from_str(&current_player).unwrap();
    deserialised.stats = stats_query.single().clone();
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    let serialised = serde_json::to_string(&deserialised).unwrap();
    map_context.load_new_region = true;
    fs::write("player_data.json", serialised).expect("Unable to write to file");
}
