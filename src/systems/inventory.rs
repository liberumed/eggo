use bevy::prelude::*;

use crate::components::*;

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
    mut ui_state: ResMut<InventoryUIState>,
    mut panel_query: Query<&mut Visibility, With<InventoryPanel>>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
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
    inventory_query: Query<&Inventory, With<Player>>,
    mut slot_query: Query<(&HotbarSlot, &mut BackgroundColor)>,
    mut count_query: Query<(&HotbarSlotCount, &mut Text)>,
) {
    let Ok(inventory) = inventory_query.single() else { return };

    for (slot, mut bg) in &mut slot_query {
        let item = inventory.get(slot.0);
        *bg = BackgroundColor(match item {
            Some(s) => get_item_color(s.item_id),
            None => Color::srgba(0.2, 0.2, 0.22, 0.9),
        });
    }

    for (slot_count, mut text) in &mut count_query {
        let item = inventory.get(slot_count.0);
        **text = match item {
            Some(s) if s.quantity > 1 => s.quantity.to_string(),
            _ => String::new(),
        };
    }
}

pub fn update_inventory_panel_ui(
    ui_state: Res<InventoryUIState>,
    inventory_query: Query<&Inventory, With<Player>>,
    mut slot_query: Query<(&InventorySlotUI, &mut BackgroundColor)>,
    mut count_query: Query<(&InventorySlotCount, &mut Text)>,
) {
    if !ui_state.open {
        return;
    }

    let Ok(inventory) = inventory_query.single() else { return };

    for (slot_ui, mut bg) in &mut slot_query {
        let item = inventory.get(slot_ui.0);
        *bg = BackgroundColor(match item {
            Some(s) => get_item_color(s.item_id),
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
}

fn get_item_color(id: ItemId) -> Color {
    match get_item_data(id).category {
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
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut player_query: Query<(&Transform, &mut Inventory), With<Player>>,
    ground_items: Query<(Entity, &Transform, &GroundItem), (With<Pickupable>, Without<Player>)>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
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
    mouse: Res<ButtonInput<MouseButton>>,
    mut inventory_query: Query<(&mut Inventory, &mut Health), With<Player>>,
    slot_query: Query<(&Interaction, &InventorySlotUI)>,
) {
    if !ui_state.open {
        return;
    }

    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok((mut inventory, mut health)) = inventory_query.single_mut() else { return };

    for (interaction, slot_ui) in &slot_query {
        if *interaction != Interaction::Hovered && *interaction != Interaction::Pressed {
            continue;
        }

        let slot_index = slot_ui.0;

        if let Some(slot) = inventory.get(slot_index) {
            let data = get_item_data(slot.item_id);

            if data.category == ItemCategory::Consumable {
                if use_consumable(slot.item_id, &mut health) {
                    inventory.remove(slot_index, 1);
                }
            }
        }
    }
}

pub fn use_hotbar_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory_query: Query<(&mut Inventory, &mut Health), With<Player>>,
) {
    let Ok((mut inventory, mut health)) = inventory_query.single_mut() else { return };

    let keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
    ];

    for (slot_index, key) in keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            if let Some(slot) = inventory.get(slot_index) {
                let data = get_item_data(slot.item_id);
                if data.category == ItemCategory::Consumable {
                    if use_consumable(slot.item_id, &mut health) {
                        inventory.remove(slot_index, 1);
                    }
                }
            }
        }
    }
}

fn use_consumable(item_id: ItemId, health: &mut Health) -> bool {
    match item_id {
        ItemId::HealthPotion => {
            if health.0 >= 10 {
                return false; // Already at full HP
            }
            health.0 = (health.0 + 5).min(10);
            true
        }
        _ => false,
    }
}

pub fn start_inventory_drag(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    ui_state: Res<InventoryUIState>,
    mut drag_state: ResMut<DragState>,
    inventory_query: Query<&Inventory, With<Player>>,
    slot_query: Query<(&Interaction, &InventorySlotUI)>,
    windows: Query<&Window>,
) {
    if !ui_state.open || drag_state.dragging_from.is_some() {
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
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
                let color = get_item_color(slot.item_id);
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
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<DragState>,
    mut inventory_query: Query<&mut Inventory, With<Player>>,
    slot_query: Query<(&Interaction, &InventorySlotUI)>,
) {
    let Some(from_slot) = drag_state.dragging_from else { return };

    if !mouse.just_released(MouseButton::Left) {
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
