use bevy::window::PrimaryWindow;

use crate::*;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<UIState>()
            .insert_resource(Inventory::default())
            .add_system(handle_keys.in_set(OnUpdate(GameState::InGame)))
            .add_system(
                draw_inventory
                    .in_schedule(OnEnter(UIState::Inventory))
                    .in_set(OnUpdate(GameState::InGame)),
            )
            .add_system(undraw_inventory.in_schedule(OnExit(UIState::Inventory)))
            .add_systems((item_button_interaction,).in_set(OnUpdate(UIState::Inventory)));
    }
}

pub struct ItemStack {
    pub item_id: usize,
    pub item_name: String,
    // item_type: ItemType,
    pub quantity: u8,
}

impl ItemStack {
    pub fn new(item_id: usize, quantity: u8) -> Self {
        Self {
            item_id,
            item_name: "".to_string(),
            quantity,
        }
    }

    pub fn add(&mut self, quantity: u8) {
        assert!(self.quantity + quantity <= 32);
        self.quantity += quantity;
    }
}

#[derive(Resource)]
pub struct Inventory {
    slots: Vec<Option<ItemStack>>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: (1..=30).map(|_| None).collect(),
        }
    }
}

impl Inventory {
    pub fn add_item(&mut self, id: usize, mut qty: u32) {
        // Distribute the quantity of items in existing slots for that item
        for slot in self.slots.iter_mut().flatten() {
            // Short circuit if all items have been distributed
            if qty == 0 {
                return;
            }
            // Try find a slot with the corresponding id
            if slot.item_id == id && slot.quantity < 32 {
                let left_in_the_slot = 32 - slot.quantity;

                if qty <= left_in_the_slot as u32 {
                    slot.quantity += qty as u8;
                    qty = 0;
                } else {
                    slot.quantity += left_in_the_slot;
                    qty -= left_in_the_slot as u32;
                }
            }
        }

        // Items have been distributed amongst corresponding existing slots
        for slot in self.slots.iter_mut() {
            // Short circuit if all the items have been distributed amongst new slots
            if qty == 0 {
                return;
            }
            if slot.is_none() {
                if qty <= 32 {
                    *slot = Some(ItemStack::new(id, qty as u8));
                    qty = 0;
                } else {
                    *slot = Some(ItemStack::new(id, 32));
                    qty -= 32;
                }
            }
        }

        // TODO: Don't purge remaining items...instead convert them into player XP
        dbg!("Remaining items to be purged: {}", qty);
    }
}

fn handle_keys(
    keys: Res<Input<KeyCode>>,
    ui_state: Res<State<UIState>>,
    mut next_ui_state: ResMut<NextState<UIState>>,
    mut gi_lock_sender: EventWriter<GlobalInteractionLockEvent>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match ui_state.0 {
            UIState::Inventory => {
                next_ui_state.set(UIState::Null);
                return;
            }
            UIState::OpenChest => {
                next_ui_state.set(UIState::Null);
                gi_lock_sender.send(GlobalInteractionLockEvent(GIState::Unlocked));
            }
            UIState::Null => {}
        }
    }

    if keys.just_pressed(KeyCode::E) {
        match ui_state.0 {
            UIState::Null => next_ui_state.set(UIState::Inventory),
            UIState::Inventory => next_ui_state.set(UIState::Null),
            _ => {}
        }
    }
}

#[derive(Component)]
struct InventoryUIRoot;

#[derive(Component, Reflect)]
struct InventoryItemButton {
    // item_type: Option<ItemType>,
    slot_idx: usize,
}

// #[derive(Component)]
// struct MiniQuantityText {
//     item_type: Option<ItemType>,
// }

fn draw_inventory(
    mut commands: Commands,
    inventory: Res<Inventory>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let mut inventory_width = primary_window.single().resolution.width() / 2.0;
    let mut inventory_height = inventory_width / (1920.0 / 1080.0);

    if inventory_height > primary_window.single().resolution.height() {
        inventory_height = primary_window.single().resolution.height() / 2.0;
        inventory_width = inventory_height * (1920.0 / 1080.0);
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
        .insert(InventoryUIRoot)
        .with_children(|commands| {
            // The background
            commands
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(inventory_width), Val::Px(inventory_height)),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceEvenly,
                        align_items: AlignItems::Center,
                        // position_type: PositionType::Absolute,
                        align_self: AlignSelf::Center,
                        margin: UiRect::left(Val::Px(
                            // Offset required for the centre of inventory width to align with centre of screen
                            (primary_window.single().resolution.width() - inventory_width) / 2.0,
                        )),
                        ..default()
                    },
                    background_color: Color::rgb(0.13, 0.14, 0.26).into(),
                    // Background: 0.13, 0.14, 0.26
                    // Box background: 0.17, 0.19, 0.36
                    // Selected: 0.55, 0.44, 0.95
                    ..default()
                })
                .insert(Name::new("Inventory Layout"))
                .with_children(|commands| {
                    // The left half
                    commands
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(45.0), Val::Percent(90.0)),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            // background_color: Color::RED.into(),
                            ..default()
                        })
                        .insert(Name::new("Left side container"))
                        .with_children(|commands| {
                            // Stats container
                            commands
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(20.0)),
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::SpaceBetween,
                                        ..default()
                                    },
                                    background_color: Color::rgba(0.17, 0.19, 0.36, 0.5).into(),
                                    ..default()
                                })
                                .insert(Name::new("Stats container"))
                                .with_children(|commands| {
                                    // Balance text
                                    commands
                                        .spawn(TextBundle {
                                            style: Style {
                                                margin: UiRect::all(Val::Percent(5.0)),
                                                ..default()
                                            },
                                            text: Text::from_section(
                                                "hi",
                                                TextStyle {
                                                    font: asset_server.load("font.otf"),
                                                    // Font size 40 looked nice on my own screen height of 2880, which is a ratio of 1:72
                                                    font_size: inventory_width / 36.0,
                                                    color: Color::WHITE,
                                                },
                                            ),
                                            ..default()
                                        })
                                        .insert(Name::new("Text"));
                                    // City centre text
                                    commands
                                        .spawn(TextBundle {
                                            style: Style {
                                                margin: UiRect::all(Val::Percent(5.0)),
                                                ..default()
                                            },
                                            text: Text::from_section(
                                                "hi2",
                                                TextStyle {
                                                    font: asset_server.load("font.otf"),
                                                    font_size: inventory_width / 36.0,
                                                    color: Color::WHITE,
                                                },
                                            ),
                                            ..default()
                                        })
                                        .insert(Name::new("Text 2"));
                                });

                            // Inventory grid container
                            commands
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(80.0)),
                                        flex_direction: FlexDirection::Row,
                                        // align_items: AlignItems::FlexEnd,
                                        justify_content: JustifyContent::SpaceAround,
                                        flex_wrap: FlexWrap::Wrap,
                                        ..default()
                                    },
                                    focus_policy: bevy::ui::FocusPolicy::Block,
                                    background_color: Color::rgb(0.17, 0.19, 0.36).into(),
                                    ..default()
                                })
                                .insert(Name::new("Inventory grid container"))
                                .with_children(|commands| {
                                    for i in 0..=29 {
                                        commands
                                            .spawn(ButtonBundle {
                                                style: Style {
                                                    size: Size::new(
                                                        Val::Percent(100.0 / (5.0 / 0.9)),
                                                        Val::Percent(100.0 / (6.0 / 0.9)),
                                                    ),
                                                    justify_content: JustifyContent::Center,
                                                    ..default()
                                                },
                                                background_color: Color::rgb(0.22, 0.25, 0.48).into(),
                                                ..default()
                                            })
                                            .insert(Name::new(format!("Button {i}")))
                                            .insert(InventoryItemButton { slot_idx: i })
                                            // .insert(InventoryItemButton {
                                            //     item_type: {
                                            //         if i < inventory.items.len() {
                                            //             Some(inventory.items[i].item_type)
                                            //         } else {
                                            //             None
                                            //         }
                                            //     },
                                            // })
                                            .with_children(|commands| {
                                                // Item icon
                                                commands.spawn(ImageBundle {
                                                    style: Style {
                                                        size: Size::new(
                                                            Val::Px(inventory_width / 17.5),
                                                            Val::Px(inventory_width / 17.5),
                                                        ),
                                                        ..default()
                                                    },
                                                    image: UiImage {
                                                        texture: {
                                                            if let Some(ref item_stack) = inventory.slots[i] {
                                                                match item_stack.item_id {
                                                                    0 => asset_server.load("items/xpdrop.png"),
                                                                    1 => asset_server.load("items/xpgem.png"),
                                                                    2 => asset_server.load("items/xpcore.png"),
                                                                    3 => asset_server.load("items/hpotS.png"),
                                                                    4 => asset_server.load("items/hpotM.png"),
                                                                    _ => unreachable!(),
                                                                }
                                                            } else {
                                                                asset_server.load("items/empty.png")
                                                            }
                                                        },
                                                        flip_x: false,
                                                        flip_y: false,
                                                    },
                                                    transform: Transform::from_scale(Vec3::splat(0.7)),
                                                    ..default()
                                                });
                                                // Quantity text
                                                commands
                                                    .spawn(TextBundle {
                                                        style: Style {
                                                            size: Size::new(Val::Percent(25.0), Val::Percent(30.0)),
                                                            position_type: PositionType::Absolute,
                                                            position: UiRect::new(
                                                                Val::Percent(5.0),
                                                                Val::Percent(0.0),
                                                                Val::Percent(65.0),
                                                                Val::Percent(0.0),
                                                            ),
                                                            ..default()
                                                        },
                                                        text: Text::from_section(
                                                            if let Some(ref item_stack) = inventory.slots[i] {
                                                                item_stack.quantity.to_string()
                                                            } else {
                                                                "/".to_string()
                                                            },
                                                            TextStyle {
                                                                font: asset_server.load("font.otf"),
                                                                font_size: inventory_width / 54.0,
                                                                color: Color::WHITE,
                                                            },
                                                        ),
                                                        ..default()
                                                    })
                                                    // .insert(MiniQuantityText {
                                                    //     item_type: {
                                                    //         if i < inventory.items.len() {
                                                    //             // Some(inventory.items[i].item_type)
                                                    //             None
                                                    //         } else {
                                                    //             None
                                                    //         }
                                                    //     },
                                                    // })
                                                    .insert(Name::new("Quantity text"));
                                                //
                                            });
                                    }
                                });
                        });

                    // Item stats container
                    commands
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(45.0), Val::Percent(90.0)),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            background_color: Color::rgb(0.17, 0.19, 0.36).into(),
                            ..default()
                        })
                        .insert(Name::new("Item stats container"));
                });
        });
}

fn undraw_inventory(mut commands: Commands, ui_root: Query<Entity, With<InventoryUIRoot>>) {
    commands.entity(ui_root.single()).despawn_recursive();
}

#[allow(clippy::complexity)]
fn item_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &InventoryItemButton),
        (Changed<Interaction>, With<InventoryItemButton>),
    >,
) {
    for (interaction, mut background_colour, item_button_cmp) in &mut interaction_query {
        match interaction {
            Interaction::Clicked => {}
            Interaction::Hovered => *background_colour = Color::rgb(0.34, 0.37, 0.60).into(),
            _ => *background_colour = Color::rgb(0.22, 0.25, 0.48).into(),
        }
    }
}
