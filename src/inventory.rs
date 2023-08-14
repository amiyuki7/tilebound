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
            .add_systems((item_button_interaction, increment_button_interaction).in_set(OnUpdate(UIState::Inventory)));
    }
}

pub struct ItemStack {
    pub item_id: usize,
    pub item_name: String,
    pub description: String,
    pub quantity: u8,
}

impl ItemStack {
    pub fn new(item_id: usize, quantity: u8) -> Self {
        Self {
            item_id,
            item_name: match item_id {
                0 => "XP Drop",
                1 => "XP Gem",
                2 => "Ultra XP Core",
                3 => "Small Health Potion",
                4 => "Medium Health Potato",
                _ => unreachable!(),
            }
            .to_string(),
            description: match item_id {
                0 => "A small drop of XP. Where'd it come from?\nGrants the player 10 XP.",
                1 => "A prized, cool looking XP gemstone.\nGrants the player 100 XP.",
                2 => "The rarest crystal of its kind.\nGrants the player 2000 XP.",
                3 => "A definitely-not-suspicious green solution!\nGrants the player 10% MAX HP.",
                4 => "Made of defeated capsules and scorpion blood.\nGrants the player 25% MAX HP.",
                _ => unreachable!(),
            }
            .to_string(),
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
    slot_idx: usize,
    is_selected: bool,
}

#[derive(Component)]
struct ItemStatsName;

#[derive(Component)]
struct ItemStatsImage;

#[derive(Component)]
struct ItemStatsDescription;

#[derive(Component)]
struct ItemStatsSelectQuantity {
    quantity: u8,
}

#[derive(Component)]
struct IncrementButton(i8);

#[derive(Component)]
struct UseButton;

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
                                    // Player Level Text
                                    commands
                                        .spawn(TextBundle {
                                            style: Style {
                                                margin: UiRect::all(Val::Percent(5.0)),
                                                ..default()
                                            },
                                            text: Text::from_section(
                                                "Player Level: -",
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
                                    // Player XP Text
                                    commands
                                        .spawn(TextBundle {
                                            style: Style {
                                                margin: UiRect::all(Val::Percent(5.0)),
                                                ..default()
                                            },
                                            text: Text::from_section(
                                                "Player XP: /",
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
                                            .insert(InventoryItemButton {
                                                slot_idx: i,
                                                is_selected: false,
                                            })
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
                                                    .insert(Name::new("Quantity text"));
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
                        .insert(Name::new("Item stats container"))
                        .with_children(|commands| {
                            // Item name
                            commands
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(10.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::top(Val::Percent(5.0)),
                                        ..default()
                                    },
                                    ..default()
                                })
                                .insert(Name::new("Item name text wrapper"))
                                .with_children(|commands| {
                                    commands
                                        .spawn(TextBundle::from_section(
                                            "",
                                            TextStyle {
                                                font: asset_server.load("font.otf"),
                                                font_size: inventory_width / 30.0,
                                                color: Color::WHITE,
                                            },
                                        ))
                                        .insert(ItemStatsName)
                                        .insert(Name::new("Item name text"));
                                });

                            // Item image
                            commands
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(50.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .insert(Name::new("Item image wrapper"))
                                .with_children(|commands| {
                                    commands
                                        .spawn(ImageBundle {
                                            image: asset_server.load("items/empty.png").into(),
                                            style: Style {
                                                size: Size::new(
                                                    Val::Px(inventory_width / 4.0),
                                                    Val::Px(inventory_width / 4.0),
                                                ),
                                                ..default()
                                            },
                                            transform: Transform::from_scale(Vec3::splat(0.8)),
                                            ..default()
                                        })
                                        .insert(ItemStatsImage)
                                        .insert(Name::new("Item image"));
                                });

                            // Item description
                            commands
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(10.0)),
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .insert(Name::new("Item quantity text wrapper"))
                                .with_children(|commands| {
                                    commands
                                        .spawn(TextBundle {
                                            style: Style {
                                                margin: UiRect::left(Val::Percent(5.0)),
                                                max_size: Size::width(Val::Percent(5.0)),
                                                ..default()
                                            },
                                            text: Text::from_section(
                                                "",
                                                TextStyle {
                                                    font: asset_server.load("font.otf"),
                                                    font_size: inventory_width / 45.0,
                                                    color: Color::WHITE,
                                                },
                                            ),
                                            ..default()
                                        })
                                        .insert(ItemStatsDescription)
                                        .insert(Name::new("Item quantity text"));
                                });

                            // Quantity selector
                            commands
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(10.0)),
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|commands| {
                                    spawn_quantity_increment_button(commands, &asset_server, -10, inventory_width);

                                    spawn_quantity_increment_button(commands, &asset_server, -1, inventory_width);

                                    // Quantity selected
                                    commands
                                        .spawn(TextBundle {
                                            style: Style {
                                                margin: UiRect::new(
                                                    Val::Percent(5.0),
                                                    Val::Percent(5.0),
                                                    Val::Percent(0.0),
                                                    Val::Percent(0.0),
                                                ),
                                                ..default()
                                            },
                                            text: Text::from_section(
                                                "/",
                                                TextStyle {
                                                    font: asset_server.load("font.otf"),
                                                    font_size: inventory_width / 30.0,
                                                    color: Color::GREEN,
                                                },
                                            ),
                                            ..default()
                                        })
                                        .insert(ItemStatsSelectQuantity { quantity: 1 });

                                    spawn_quantity_increment_button(commands, &asset_server, 1, inventory_width);

                                    spawn_quantity_increment_button(commands, &asset_server, 10, inventory_width);
                                });

                            // Use button
                            commands
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(20.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|commands| {
                                    commands
                                        .spawn(ButtonBundle {
                                            style: Style {
                                                size: Size::new(Val::Percent(30.0), Val::Percent(50.0)),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            background_color: Color::rgb(0.22, 0.25, 0.48).into(),
                                            ..default()
                                        })
                                        .insert(UseButton)
                                        .insert(Name::new("Use button"))
                                        .with_children(|commands| {
                                            commands.spawn(TextBundle::from_section(
                                                "USE",
                                                TextStyle {
                                                    font: asset_server.load("font.otf"),
                                                    font_size: inventory_width / 30.0,
                                                    color: Color::WHITE,
                                                },
                                            ));
                                        });
                                });
                        });
                });
        });
}

fn undraw_inventory(mut commands: Commands, ui_root: Query<Entity, With<InventoryUIRoot>>) {
    commands.entity(ui_root.single()).despawn_recursive();
}

#[allow(clippy::complexity)]
fn item_button_interaction(
    mut item_buttons: ParamSet<(
        Query<(&mut InventoryItemButton, &mut BackgroundColor)>,
        Query<(&Interaction, &mut BackgroundColor, &mut InventoryItemButton), Changed<Interaction>>,
    )>,
    mut texts: ParamSet<(
        Query<&mut Text, With<ItemStatsName>>,
        Query<&mut Text, With<ItemStatsDescription>>,
        Query<&mut Text, With<ItemStatsSelectQuantity>>,
    )>,
    mut select_quantity: Query<&mut ItemStatsSelectQuantity>,
    mut image: Query<&mut UiImage, With<ItemStatsImage>>,
    inventory: Res<Inventory>,
    asset_server: Res<AssetServer>,
) {
    let mut selected_slot_idx: Option<usize> = None;

    for (interaction, mut background_colour, mut item_button_cmp) in &mut item_buttons.p1() {
        match interaction {
            Interaction::Clicked => {
                let slot = &inventory.slots[item_button_cmp.slot_idx];
                if let Some(item_stack) = slot {
                    texts.p0().get_single_mut().unwrap().sections[0].value = item_stack.item_name.clone();
                    texts.p1().get_single_mut().unwrap().sections[0].value = item_stack.description.clone();
                    texts.p2().get_single_mut().unwrap().sections[0].value = "1".to_string();
                    select_quantity.get_single_mut().unwrap().quantity = 1;
                    image.get_single_mut().unwrap().texture = match item_stack.item_id {
                        0 => asset_server.load("items/xpdrop.png"),
                        1 => asset_server.load("items/xpgem.png"),
                        2 => asset_server.load("items/xpcore.png"),
                        3 => asset_server.load("items/hpotS.png"),
                        4 => asset_server.load("items/hpotM.png"),
                        _ => unreachable!(),
                    };

                    selected_slot_idx = Some(item_button_cmp.slot_idx);
                    item_button_cmp.is_selected = true;
                    *background_colour = Color::rgb(0.06, 0.08, 0.17).into()
                }
            }
            Interaction::Hovered => {
                if !item_button_cmp.is_selected {
                    *background_colour = Color::rgb(0.34, 0.37, 0.60).into()
                }
            }
            _ => {
                if !item_button_cmp.is_selected {
                    *background_colour = Color::rgb(0.22, 0.25, 0.48).into()
                }
            }
        }
    }

    if let Some(slot_idx) = selected_slot_idx {
        for (mut item_button, mut background_colour) in &mut item_buttons.p0() {
            if item_button.slot_idx != slot_idx {
                item_button.is_selected = false;
                *background_colour = Color::rgb(0.22, 0.25, 0.48).into()
            }
        }
    }
}

fn spawn_quantity_increment_button(
    commands: &mut ChildBuilder,
    asset_server: &AssetServer,
    amount: i8,
    base_width: f32,
) {
    commands
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Percent(10.0), Val::Percent(50.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::new(
                    Val::Percent(2.0),
                    Val::Percent(2.0),
                    Val::Percent(0.0),
                    Val::Percent(0.0),
                ),
                ..default()
            },
            background_color: Color::rgb(0.13, 0.14, 0.26).into(),
            ..default()
        })
        .insert(IncrementButton(amount))
        .with_children(|commands| {
            commands.spawn(TextBundle::from_section(
                format!("{}{}", if amount.is_positive() { "+" } else { "-" }, amount.abs()),
                TextStyle {
                    font: asset_server.load("font.otf"),
                    font_size: base_width / 45.0,
                    color: Color::WHITE,
                },
            ));
        });
}

fn increment_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &IncrementButton),
        (Changed<Interaction>, With<IncrementButton>),
    >,
    mut select_quantity: Query<(&mut ItemStatsSelectQuantity, &mut Text)>,
    item_buttons: Query<&InventoryItemButton>,
    inventory: Res<Inventory>,
) {
    let mut target_stack_qty = 0;

    for item_button in &item_buttons {
        if item_button.is_selected {
            if let Some(ref item_stack) = inventory.slots[item_button.slot_idx] {
                target_stack_qty = item_stack.quantity;
            } else {
                panic!("Selected item slot must exist in the inventory resource. Something is broken... fix it!");
            }
        }
    }

    if target_stack_qty == 0 {
        return;
    }

    for (interaction, mut background_colour, IncrementButton(amount)) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                for (mut qty_comp, mut qty_text) in &mut select_quantity {
                    // Min: 1
                    // Max: target_stack_qty
                    let mut net = 1u8;

                    if amount.is_positive() {
                        net = qty_comp.quantity + *amount as u8;
                        if net > target_stack_qty {
                            net = target_stack_qty;
                        }
                    } else {
                        let abs_decrement = amount.unsigned_abs();
                        if abs_decrement < qty_comp.quantity {
                            net = qty_comp.quantity - abs_decrement;
                        }
                    }

                    qty_comp.quantity = net;
                    qty_text.sections[0].value = net.to_string();
                }
            }
            Interaction::Hovered => *background_colour = Color::rgb(0.25, 0.26, 0.38).into(),
            _ => *background_colour = Color::rgb(0.13, 0.14, 0.26).into(),
        }
    }
}
