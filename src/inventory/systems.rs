use bevy::prelude::*;

use super::weapons::{PlayerWeapon, Weapon};
use crate::core::{GameAction, Health, InputBindings};
use crate::player::Player;
use super::{get_weapon_stats, ConsumableEffect, EquippedWeaponId, GroundItem, GroundItemBob, Inventory, InventorySlot, ItemCategory, ItemIcons, ItemId, ItemRegistry, Pickupable};
use crate::ui::{
    HotbarSlot, HotbarSlotCount, HotbarSlotIcon,
    InventoryPanel, InventorySlotCount, InventorySlotIcon, InventorySlotUI,
};

#[derive(Resource, Default)]
pub struct InventoryUIState {
    pub open: bool,
}

/// Tracks if the cursor is currently over any UI element
#[derive(Resource, Default)]
pub struct CursorOverUI(pub bool);

/// Tracks drag and drop state for inventory
#[derive(Resource, Default)]
pub struct DragState {
    pub dragging_from: Option<usize>,
    pub drag_visual: Option<Entity>,
}

/// Tracks which hotbar slot is selected for weapons (None = fists)
#[derive(Resource)]
pub struct SelectedHotbarSlot(pub Option<usize>);

impl Default for SelectedHotbarSlot {
    fn default() -> Self {
        Self(Some(0))  // Start with slot 0 selected (sword)
    }
}

/// Marker for the dragged item visual
#[derive(Component)]
pub struct DraggedItemVisual;

/// System that updates CursorOverUI resource - run this before other input systems
pub fn update_cursor_over_ui(
    interaction_query: Query<&Interaction, With<Node>>,
    mut cursor_over_ui: ResMut<CursorOverUI>,
) {
    cursor_over_ui.0 = interaction_query
        .iter()
        .any(|i| *i == Interaction::Hovered || *i == Interaction::Pressed);
}

/// Run condition: returns true only when cursor is NOT over UI
pub fn cursor_not_over_ui(cursor_over_ui: Res<CursorOverUI>) -> bool {
    !cursor_over_ui.0
}

pub fn toggle_inventory(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    mut ui_state: ResMut<InventoryUIState>,
    mut panel_query: Query<&mut Visibility, With<InventoryPanel>>,
) {
    if bindings.just_pressed(GameAction::ToggleInventory, &keyboard, &mouse) {
        ui_state.open = !ui_state.open;
        if let Ok(mut visibility) = panel_query.single_mut() {
            *visibility = if ui_state.open {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub fn update_hotbar_ui(
    registry: Res<ItemRegistry>,
    item_icons: Res<ItemIcons>,
    selected_slot: Res<SelectedHotbarSlot>,
    inventory_query: Query<&Inventory, With<Player>>,
    mut slot_query: Query<(&HotbarSlot, &mut BackgroundColor, &mut BorderColor)>,
    mut count_query: Query<(&HotbarSlotCount, &mut Text)>,
    mut icon_query: Query<(&HotbarSlotIcon, &mut ImageNode, &mut Visibility)>,
) {
    let Ok(inventory) = inventory_query.single() else { return };

    for (slot, mut bg, mut border) in &mut slot_query {
        let item = inventory.get(slot.0);
        *bg = BackgroundColor(match item {
            Some(s) => {
                if item_icons.icons.contains_key(&s.item_id) {
                    Color::srgba(0.15, 0.15, 0.17, 0.9)
                } else {
                    get_item_color(&registry, s.item_id)
                }
            }
            None => Color::srgba(0.2, 0.2, 0.22, 0.9),
        });

        // Green border for selected weapon slot
        *border = if selected_slot.0 == Some(slot.0) {
            BorderColor::all(Color::srgb(0.2, 0.8, 0.3))
        } else {
            BorderColor::all(Color::srgb(0.4, 0.4, 0.45))
        };
    }

    for (slot_count, mut text) in &mut count_query {
        let item = inventory.get(slot_count.0);
        **text = match item {
            Some(s) if s.quantity > 1 => s.quantity.to_string(),
            _ => String::new(),
        };
    }

    for (slot_icon, mut image_node, mut visibility) in &mut icon_query {
        let item = inventory.get(slot_icon.0);
        if let Some(slot) = item {
            if let Some(icon_handle) = item_icons.icons.get(&slot.item_id) {
                image_node.image = icon_handle.clone();
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn update_inventory_panel_ui(
    ui_state: Res<InventoryUIState>,
    registry: Res<ItemRegistry>,
    item_icons: Res<ItemIcons>,
    inventory_query: Query<&Inventory, With<Player>>,
    mut slot_query: Query<(&InventorySlotUI, &mut BackgroundColor)>,
    mut count_query: Query<(&InventorySlotCount, &mut Text)>,
    mut icon_query: Query<(&InventorySlotIcon, &mut ImageNode, &mut Visibility)>,
) {
    if !ui_state.open {
        // Hide all inventory icons when panel is closed
        for (_, _, mut visibility) in &mut icon_query {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let Ok(inventory) = inventory_query.single() else { return };

    for (slot_ui, mut bg) in &mut slot_query {
        let item = inventory.get(slot_ui.0);
        *bg = BackgroundColor(match item {
            Some(s) => {
                if item_icons.icons.contains_key(&s.item_id) {
                    Color::srgba(0.15, 0.15, 0.17, 1.0)
                } else {
                    get_item_color(&registry, s.item_id)
                }
            }
            None => Color::srgba(0.2, 0.2, 0.22, 1.0),
        });
    }

    for (slot_count, mut text) in &mut count_query {
        let item = inventory.get(slot_count.0);
        **text = match item {
            Some(s) if s.quantity > 1 => s.quantity.to_string(),
            _ => String::new(),
        };
    }

    for (slot_icon, mut image_node, mut visibility) in &mut icon_query {
        let item = inventory.get(slot_icon.0);
        if let Some(slot) = item {
            if let Some(icon_handle) = item_icons.icons.get(&slot.item_id) {
                image_node.image = icon_handle.clone();
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn get_item_color(registry: &ItemRegistry, id: ItemId) -> Color {
    let category = registry
        .items
        .get(&id)
        .map(|item| item.category)
        .unwrap_or(ItemCategory::Consumable);
    match category {
        ItemCategory::Weapon => Color::srgba(0.6, 0.4, 0.3, 1.0),
        ItemCategory::Armor => Color::srgba(0.4, 0.5, 0.6, 1.0),
        ItemCategory::Consumable => Color::srgba(0.4, 0.6, 0.4, 1.0),
    }
}

pub fn animate_ground_items(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut GroundItemBob), With<GroundItem>>,
) {
    for (mut transform, mut bob) in &mut query {
        bob.phase += time.delta_secs() * 3.0;
        // Subtle bobbing motion
        transform.translation.z = -0.3 + bob.phase.sin() * 0.05;
    }
}

pub fn hover_ground_items(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    player_query: Query<&Transform, With<Player>>,
    mut ground_items: Query<(&Transform, &mut GroundItemBob), (With<GroundItem>, Without<Player>)>,
) {
    // Reset all hover states first
    for (_, mut bob) in &mut ground_items {
        bob.hovered = false;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(player_transform) = player_query.single() else { return };

    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);
    let hover_radius = 20.0;
    let max_reach = 50.0;

    // Only show hover if within reach
    if world_pos.distance(player_pos) > max_reach {
        return;
    }

    for (item_transform, mut bob) in &mut ground_items {
        let item_pos = Vec2::new(item_transform.translation.x, item_transform.translation.y);
        if world_pos.distance(item_pos) < hover_radius {
            bob.hovered = true;
            return; // Only hover one item
        }
    }
}

pub fn apply_ground_item_hover(
    mut query: Query<(&mut Transform, &GroundItemBob), With<GroundItem>>,
) {
    for (mut transform, bob) in &mut query {
        let target_scale = if bob.hovered { 1.3 } else { 1.0 };
        // Smooth transition
        let current = transform.scale.x;
        let new_scale = current + (target_scale - current) * 0.2;
        transform.scale = Vec3::splat(new_scale);
    }
}

pub fn pickup_ground_items(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut player_query: Query<(&Transform, &mut Inventory), With<Player>>,
    ground_items: Query<(Entity, &Transform, &GroundItem), (With<Pickupable>, Without<Player>)>,
) {
    if !bindings.just_pressed(GameAction::InventoryPickup, &keyboard, &mouse) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok((player_transform, mut inventory)) = player_query.single_mut() else { return };

    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);
    let pickup_radius = 20.0;
    let max_reach = 50.0;

    // Check if clicked position is within reach
    if world_pos.distance(player_pos) > max_reach {
        return;
    }

    for (entity, item_transform, ground_item) in &ground_items {
        let item_pos = Vec2::new(item_transform.translation.x, item_transform.translation.y);
        if world_pos.distance(item_pos) < pickup_radius {
            if inventory.try_add(ground_item.item_id, ground_item.quantity) {
                commands.entity(entity).despawn();
            }
            return;
        }
    }
}

pub fn handle_inventory_right_click(
    ui_state: Res<InventoryUIState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    registry: Res<ItemRegistry>,
    mut player_query: Query<(&mut Inventory, &mut Health, &mut EquippedWeaponId), With<Player>>,
    mut weapon_query: Query<&mut Weapon, With<PlayerWeapon>>,
    slot_query: Query<(&Interaction, &InventorySlotUI)>,
) {
    if !ui_state.open {
        return;
    }

    if !bindings.just_pressed(GameAction::InventoryUse, &keyboard, &mouse) {
        return;
    }

    let Ok((mut inventory, mut health, mut equipped)) = player_query.single_mut() else { return };

    for (interaction, slot_ui) in &slot_query {
        if *interaction != Interaction::Hovered && *interaction != Interaction::Pressed {
            continue;
        }

        let slot_index = slot_ui.0;

        if let Some(slot) = inventory.get(slot_index) {
            let Some(item) = registry.items.get(&slot.item_id) else { continue };

            match item.category {
                ItemCategory::Consumable => {
                    if use_consumable(&registry, slot.item_id, &mut health) {
                        inventory.remove(slot_index, 1);
                    }
                }
                ItemCategory::Weapon => {
                    // Equip weapon: swap with currently equipped
                    if let Some(new_weapon_stats) = get_weapon_stats(slot.item_id, &mut meshes, &mut materials) {
                        let old_weapon_id = equipped.0;

                        // Update weapon entity stats
                        if let Ok(mut weapon) = weapon_query.single_mut() {
                            *weapon = new_weapon_stats;
                        }

                        // Update equipped weapon id
                        equipped.0 = slot.item_id;

                        // Put old weapon in inventory slot
                        inventory.slots[slot_index] = Some(InventorySlot {
                            item_id: old_weapon_id,
                            quantity: 1,
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn use_hotbar_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    registry: Res<ItemRegistry>,
    mut selected_slot: ResMut<SelectedHotbarSlot>,
    mut player_query: Query<(&mut Inventory, &mut Health), With<Player>>,
) {
    let Ok((mut inventory, mut health)) = player_query.single_mut() else { return };

    let actions = [
        GameAction::Hotbar1,
        GameAction::Hotbar2,
        GameAction::Hotbar3,
        GameAction::Hotbar4,
        GameAction::Hotbar5,
    ];

    for (slot_index, action) in actions.iter().enumerate() {
        if bindings.just_pressed(*action, &keyboard, &mouse) {
            if let Some(slot) = inventory.get(slot_index) {
                let Some(item) = registry.items.get(&slot.item_id) else { continue };
                match item.category {
                    ItemCategory::Consumable => {
                        if use_consumable(&registry, slot.item_id, &mut health) {
                            inventory.remove(slot_index, 1);
                        }
                    }
                    ItemCategory::Weapon => {
                        // Toggle selection: if already selected, deselect (use fists)
                        if selected_slot.0 == Some(slot_index) {
                            selected_slot.0 = None;
                        } else {
                            selected_slot.0 = Some(slot_index);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Syncs weapon stats based on selected hotbar slot (or fists if none selected)
pub fn sync_selected_weapon(
    selected_slot: Res<SelectedHotbarSlot>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    inventory_query: Query<&Inventory, With<Player>>,
    mut equipped_query: Query<&mut EquippedWeaponId, With<Player>>,
    mut weapon_query: Query<&mut Weapon, With<PlayerWeapon>>,
) {
    if !selected_slot.is_changed() {
        return;
    }

    let Ok(inventory) = inventory_query.single() else { return };
    let Ok(mut equipped) = equipped_query.single_mut() else { return };
    let Ok(mut weapon) = weapon_query.single_mut() else { return };

    match selected_slot.0 {
        Some(slot_index) => {
            // Get weapon from selected slot
            if let Some(slot) = inventory.get(slot_index) {
                if let Some(new_weapon) = get_weapon_stats(slot.item_id, &mut meshes, &mut materials) {
                    *weapon = new_weapon;
                    equipped.0 = slot.item_id;
                }
            }
        }
        None => {
            // Use fists
            if let Some(fist_weapon) = get_weapon_stats(ItemId::Fist, &mut meshes, &mut materials) {
                *weapon = fist_weapon;
                equipped.0 = ItemId::Fist;
            }
        }
    }
}

fn use_consumable(registry: &ItemRegistry, item_id: ItemId, health: &mut Health) -> bool {
    let Some(item) = registry.items.get(&item_id) else { return false };

    match &item.consumable_effect {
        Some(ConsumableEffect::Heal(amount)) => {
            if health.0 >= 10 {
                return false; // Already at full HP
            }
            health.0 = (health.0 + *amount).min(10);
            true
        }
        None => false,
    }
}

pub fn start_inventory_drag(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    ui_state: Res<InventoryUIState>,
    registry: Res<ItemRegistry>,
    mut drag_state: ResMut<DragState>,
    inventory_query: Query<&Inventory, With<Player>>,
    slot_query: Query<(&Interaction, &InventorySlotUI)>,
    windows: Query<&Window>,
) {
    if !ui_state.open || drag_state.dragging_from.is_some() {
        return;
    }

    if !bindings.just_pressed(GameAction::InventoryPickup, &keyboard, &mouse) {
        return;
    }

    let Ok(inventory) = inventory_query.single() else { return };
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };

    for (interaction, slot_ui) in &slot_query {
        if *interaction == Interaction::Pressed || *interaction == Interaction::Hovered {
            let slot_index = slot_ui.0;
            if let Some(slot) = inventory.get(slot_index) {
                // Start dragging
                drag_state.dragging_from = Some(slot_index);

                // Spawn drag visual
                let color = get_item_color(&registry, slot.item_id);
                let visual = commands.spawn((
                    DraggedItemVisual,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(cursor_pos.x - 20.0),
                        top: Val::Px(cursor_pos.y - 20.0),
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        ..default()
                    },
                    BackgroundColor(color.with_alpha(0.8)),
                    BorderRadius::all(Val::Px(4.0)),
                    GlobalZIndex(100),
                )).id();
                drag_state.drag_visual = Some(visual);
                return;
            }
        }
    }
}

pub fn update_drag_visual(
    drag_state: Res<DragState>,
    windows: Query<&Window>,
    mut visual_query: Query<&mut Node, With<DraggedItemVisual>>,
) {
    if drag_state.dragging_from.is_none() {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };

    for mut node in &mut visual_query {
        node.left = Val::Px(cursor_pos.x - 20.0);
        node.top = Val::Px(cursor_pos.y - 20.0);
    }
}

pub fn end_inventory_drag(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    mut drag_state: ResMut<DragState>,
    mut inventory_query: Query<&mut Inventory, With<Player>>,
    slot_query: Query<(&Interaction, &InventorySlotUI)>,
) {
    let Some(from_slot) = drag_state.dragging_from else { return };

    if !bindings.just_released(GameAction::InventoryPickup, &keyboard, &mouse) {
        return;
    }

    // Clean up drag visual
    if let Some(visual) = drag_state.drag_visual.take() {
        commands.entity(visual).despawn();
    }
    drag_state.dragging_from = None;

    let Ok(mut inventory) = inventory_query.single_mut() else { return };

    // Find target slot
    for (interaction, slot_ui) in &slot_query {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            let to_slot = slot_ui.0;
            if to_slot != from_slot {
                // Swap items
                inventory.swap(from_slot, to_slot);
            }
            return;
        }
    }
}
