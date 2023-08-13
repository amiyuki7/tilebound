use bevy::window::PrimaryWindow;

use crate::*;

pub struct ChestPlugin;

impl Plugin for ChestPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChestOpenEvent>()
            .add_system(on_chest_open)
            .add_system(undraw_chest_ui.in_schedule(OnExit(UIState::OpenChest)));
    }
}

#[derive(Component, Serialize, Deserialize, FromReflect, Reflect, Default, Debug, Clone)]
pub struct Chest {
    pub hex_coord: HexCoord,
    /// (Item ID, Item Count)
    pub contents: Vec<(usize, u32)>,
}

pub struct ChestOpenEvent {
    pub chest_ent: Entity,
    pub chest: Chest,
}

#[derive(Component)]
struct ChestUIRoot;

pub fn on_chest_open(
    mut commands: Commands,
    mut chest_open_event: EventReader<ChestOpenEvent>,
    mut next_ui_state: ResMut<NextState<UIState>>,
    mut gi_lock_sender: EventWriter<GlobalInteractionLockEvent>,
    mut map_ctx: ResMut<MapContext>,
    mut player: Query<&mut Player>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    for event in &mut chest_open_event {
        next_ui_state.set(UIState::OpenChest);
        gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Locked));

        player.get_single_mut().unwrap().path = None;

        commands.entity(event.chest_ent).despawn_recursive();

        let chest = &event.chest;

        assert!(!chest.contents.is_empty(), "Chest must contain at least 1 item");
        assert!(chest.contents.len() <= 5, "Chest can contain at most 5 items");

        map_ctx.remove_chest(chest.hex_coord);
        let mut ui_width = primary_window.single().resolution.width() / 2.0;
        let mut ui_height = ui_width / (1920.0 / 1080.0) / 2.0;

        if ui_height > primary_window.single().resolution.height() {
            ui_height = primary_window.single().resolution.height() / 2.0 / 2.0;
            ui_width = ui_height * (1920.0 / 1080.0);
        }

        commands
            .spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    position_type: PositionType::Absolute,

                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.9).into(),
                ..default()
            })
            .insert(Name::new("InventoryUIRoot"))
            .insert(ChestUIRoot)
            .with_children(|commands| {
                // A "sub container"
                commands
                    .spawn(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(ui_width), Val::Px(ui_height)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            align_self: AlignSelf::Center,
                            margin: UiRect::left(Val::Px(
                                // Offset required for the centre of inventory width to align with centre of screen
                                (primary_window.single().resolution.width() - ui_width) / 2.0,
                            )),
                            ..default()
                        },
                        background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(),
                        ..default()
                    })
                    .with_children(|commands| {
                        // Top section
                        commands
                            .spawn(NodeBundle {
                                style: Style {
                                    size: Size::new(Val::Percent(100.0), Val::Percent(70.0)),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|commands| {
                                for i in 0..chest.contents.len() {
                                    commands
                                        .spawn(NodeBundle {
                                            style: Style {
                                                size: Size::new(Val::Percent(20.0), Val::Percent(100.0)),
                                                flex_direction: FlexDirection::Column,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::End,
                                                ..default()
                                            },
                                            ..default()
                                        })
                                        .with_children(|commands| {
                                            commands
                                                .spawn(NodeBundle {
                                                    style: Style {
                                                        size: Size::new(Val::Percent(50.0), Val::Percent(50.0)),
                                                        align_items: AlignItems::Center,
                                                        justify_content: JustifyContent::Center,
                                                        ..default()
                                                    },
                                                    // background_color: Color::WHITE.into(),
                                                    ..default()
                                                })
                                                .with_children(|commands| {
                                                    commands.spawn(ImageBundle {
                                                        style: Style {
                                                            size: Size::new(
                                                                Val::Px(ui_width / 10.0),
                                                                Val::Px(ui_width / 10.0),
                                                            ),
                                                            ..default()
                                                        },
                                                        image: UiImage {
                                                            texture: match chest.contents[i].0 {
                                                                0 => asset_server.load("items/xpdrop.png"),
                                                                1 => asset_server.load("items/xpgem.png"),
                                                                2 => asset_server.load("items/xpcore.png"),
                                                                3 => asset_server.load("items/hpotS.png"),
                                                                4 => asset_server.load("items/hpotM.png"),
                                                                _ => unreachable!(),
                                                            },
                                                            ..default()
                                                        },
                                                        ..default()
                                                    });
                                                });
                                            commands
                                                .spawn(NodeBundle {
                                                    style: Style {
                                                        size: Size::new(Val::Percent(50.0), Val::Percent(20.0)),
                                                        align_items: AlignItems::Center,
                                                        justify_content: JustifyContent::Center,
                                                        margin: UiRect::top(Val::Percent(4.0)),
                                                        ..default()
                                                    },
                                                    // background_color: Color::BLUE.into(),
                                                    ..default()
                                                })
                                                .with_children(|commands| {
                                                    commands.spawn(TextBundle {
                                                        text: Text::from_section(
                                                            chest.contents[i].1.to_string(),
                                                            TextStyle {
                                                                font: asset_server.load("font.otf"),
                                                                font_size: ui_width / 25.0,
                                                                color: Color::WHITE,
                                                            },
                                                        ),
                                                        ..default()
                                                    });
                                                });
                                        });
                                }
                            });

                        // Bottom section
                        commands
                            .spawn(NodeBundle {
                                style: Style {
                                    size: Size::new(Val::Percent(100.0), Val::Percent(30.0)),
                                    align_items: AlignItems::Start,
                                    justify_content: JustifyContent::Center,
                                    margin: UiRect::top(Val::Percent(2.0)),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|commands| {
                                commands.spawn(TextBundle {
                                    text: Text::from_section(
                                        "Press [ESC] to continue",
                                        TextStyle {
                                            font: asset_server.load("font.otf"),
                                            font_size: ui_width / 26.0,
                                            color: Color::rgba(1.0, 1.0, 1.0, 0.6),
                                        },
                                    ),
                                    style: Style { ..default() },
                                    ..default()
                                });
                            });
                    });
            });
    }
}

fn undraw_chest_ui(mut commands: Commands, ui_root: Query<Entity, With<ChestUIRoot>>) {
    commands.entity(ui_root.single()).despawn_recursive();
}
