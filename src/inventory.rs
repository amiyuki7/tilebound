use crate::*;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<UIState>()
            .insert_resource(Inventory::default())
            .add_system(handle_keys.in_set(OnUpdate(GameState::InGame)));
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
}
