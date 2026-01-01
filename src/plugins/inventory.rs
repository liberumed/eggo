use bevy::prelude::*;

use crate::resources::GameState;
use crate::systems::{
    animate_ground_items, apply_ground_item_hover, cursor_not_over_ui, end_inventory_drag,
    handle_inventory_right_click, hover_ground_items, pickup_ground_items, start_inventory_drag,
    toggle_inventory, update_cursor_over_ui, update_drag_visual, update_hotbar_ui,
    update_inventory_panel_ui, use_hotbar_keys, CursorOverUI, DragState, InventoryUIState,
};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InventoryUIState>()
            .init_resource::<CursorOverUI>()
            .init_resource::<DragState>()
            .add_systems(
                Update,
                (
                    // Update cursor state first
                    update_cursor_over_ui,
                    toggle_inventory,
                    update_hotbar_ui,
                    update_inventory_panel_ui,
                    handle_inventory_right_click,
                    use_hotbar_keys,
                    animate_ground_items,
                    hover_ground_items,
                    apply_ground_item_hover,
                    // Drag and drop
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
