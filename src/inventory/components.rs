#![allow(dead_code)]

use bevy::prelude::*;

use super::data::ItemId;

pub const INVENTORY_SIZE: usize = 10;
pub const HOTBAR_SIZE: usize = 5;

#[derive(Clone)]
pub struct InventorySlot {
    pub item_id: ItemId,
    pub quantity: u32,
}

#[derive(Component)]
pub struct Inventory {
    pub slots: [Option<InventorySlot>; INVENTORY_SIZE],
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: Default::default(),
        }
    }
}

impl Inventory {
    pub fn try_add(&mut self, item_id: ItemId, quantity: u32) -> bool {
        use super::data::get_item_data;
        let data = get_item_data(item_id);

        // Try to stack with existing
        for slot in self.slots.iter_mut() {
            if let Some(ref mut s) = slot {
                if s.item_id == item_id && s.quantity < data.stack_max {
                    let can_add = (data.stack_max - s.quantity).min(quantity);
                    s.quantity += can_add;
                    if can_add == quantity {
                        return true;
                    }
                }
            }
        }

        // Find empty slot
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some(InventorySlot { item_id, quantity });
                return true;
            }
        }

        false
    }

    pub fn remove(&mut self, index: usize, quantity: u32) -> Option<(ItemId, u32)> {
        if let Some(ref mut slot) = self.slots[index] {
            let removed = slot.quantity.min(quantity);
            slot.quantity -= removed;
            let id = slot.item_id;
            if slot.quantity == 0 {
                self.slots[index] = None;
            }
            return Some((id, removed));
        }
        None
    }

    pub fn get(&self, index: usize) -> Option<&InventorySlot> {
        self.slots[index].as_ref()
    }

    pub fn hotbar_slots(&self) -> &[Option<InventorySlot>] {
        &self.slots[..HOTBAR_SIZE]
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.slots.swap(a, b);
    }
}

#[derive(Component)]
pub struct GroundItem {
    pub item_id: ItemId,
    pub quantity: u32,
}

#[derive(Component, Default)]
pub struct GroundItemBob {
    pub phase: f32,
    pub hovered: bool,
}

#[derive(Component)]
pub struct Pickupable;

#[derive(Component)]
pub struct Armor {
    pub name: String,
    pub defense: i32,
    pub rarity: crate::combat::Rarity,
}

#[derive(Component)]
pub struct Consumable {
    pub name: String,
    pub heal_amount: i32,
}

pub mod item_catalog {
    use super::*;
    use crate::combat::Rarity;

    pub fn leather_armor() -> Armor {
        Armor {
            name: "Leather Armor".to_string(),
            defense: 2,
            rarity: Rarity::Common,
        }
    }

    pub fn health_potion() -> Consumable {
        Consumable {
            name: "Health Potion".to_string(),
            heal_amount: 5,
        }
    }
}
