use bevy::prelude::*;

use super::Rarity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemId {
    RustyKnife,
    Fist,
    HealthPotion,
    LeatherArmor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Consumable,
}

#[derive(Clone)]
pub struct ItemData {
    pub id: ItemId,
    pub name: String,
    pub category: ItemCategory,
    pub stack_max: u32,
}

#[derive(Clone)]
pub struct InventorySlot {
    pub item_id: ItemId,
    pub quantity: u32,
}

pub const INVENTORY_SIZE: usize = 10;
pub const HOTBAR_SIZE: usize = 5;

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

pub fn get_item_data(id: ItemId) -> ItemData {
    match id {
        ItemId::RustyKnife => ItemData {
            id,
            name: "Rusty Knife".to_string(),
            category: ItemCategory::Weapon,
            stack_max: 1,
        },
        ItemId::Fist => ItemData {
            id,
            name: "Fist".to_string(),
            category: ItemCategory::Weapon,
            stack_max: 1,
        },
        ItemId::HealthPotion => ItemData {
            id,
            name: "Health Potion".to_string(),
            category: ItemCategory::Consumable,
            stack_max: 10,
        },
        ItemId::LeatherArmor => ItemData {
            id,
            name: "Leather Armor".to_string(),
            category: ItemCategory::Armor,
            stack_max: 1,
        },
    }
}

#[derive(Component)]
pub struct Armor {
    pub name: String,
    pub defense: i32,
    pub rarity: Rarity,
}

#[derive(Component)]
pub struct Consumable {
    pub name: String,
    pub heal_amount: i32,
}

pub mod item_catalog {
    use super::*;

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
