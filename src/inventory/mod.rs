pub mod components;
pub mod systems;

pub use components::*;
pub use systems::*;

use bevy::prelude::*;

use crate::core::GameState;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InventoryUIState>()
            .init_resource::<CursorOverUI>()
            .init_resource::<DragState>()
            .init_resource::<SelectedHotbarSlot>()
            .add_systems(
                Update,
                (
                    update_cursor_over_ui,
                    toggle_inventory,
                    update_hotbar_ui,
                    update_inventory_panel_ui,
                    handle_inventory_right_click,
                    use_hotbar_keys,
                    sync_selected_weapon,
                    animate_ground_items,
                    hover_ground_items,
                    apply_ground_item_hover,
                    start_inventory_drag,
                    update_drag_visual,
                    end_inventory_drag,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                pickup_ground_items
                    .run_if(in_state(GameState::Playing))
                    .run_if(cursor_not_over_ui),
            );
    }
}
