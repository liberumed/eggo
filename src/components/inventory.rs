use std::collections::HashMap;

use bevy::prelude::*;

use super::{Rarity, Weapon};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemId {
    WoodenStick,
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

/// Visual representation of an item when on the ground
#[derive(Clone)]
pub struct GroundItemVisual {
    /// Each tuple: (mesh, material, offset position)
    pub meshes: Vec<(Handle<Mesh>, Handle<ColorMaterial>, Vec3)>,
}

/// Effect when a consumable is used
#[derive(Clone)]
pub enum ConsumableEffect {
    Heal(i32),
}

/// Complete definition of an item - single source of truth
#[derive(Clone)]
pub struct ItemDefinition {
    pub name: String,
    pub category: ItemCategory,
    pub stack_max: u32,
    pub ground_visual: GroundItemVisual,
    pub weapon: Option<Weapon>,
    pub consumable_effect: Option<ConsumableEffect>,
}

/// Registry of all items - loaded at startup
#[derive(Resource)]
pub struct ItemRegistry {
    pub items: HashMap<ItemId, ItemDefinition>,
}

/// Simple item data for backward compatibility (will be removed)
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

pub fn get_item_data(id: ItemId) -> ItemData {
    match id {
        ItemId::WoodenStick => ItemData {
            id,
            name: "Wooden Stick".to_string(),
            category: ItemCategory::Weapon,
            stack_max: 1,
        },
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

/// Returns weapon stats for a weapon ItemId, or None if not a weapon
pub fn get_weapon_stats(
    id: ItemId,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> Option<super::Weapon> {
    use super::catalog;
    match id {
        ItemId::WoodenStick => Some(catalog::wooden_stick(meshes, materials)),
        ItemId::RustyKnife => Some(catalog::rusty_knife(meshes, materials)),
        ItemId::Fist => Some(catalog::fist(meshes, materials)),
        _ => None,
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

/// Builds the item registry with all item definitions
pub fn build_item_registry(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> ItemRegistry {
    use super::{AttackType, DamageType, WeaponVisual};

    let mut items = HashMap::new();

    // Wooden Stick
    items.insert(
        ItemId::WoodenStick,
        ItemDefinition {
            name: "Wooden Stick".to_string(),
            category: ItemCategory::Weapon,
            stack_max: 1,
            ground_visual: GroundItemVisual {
                meshes: vec![(
                    meshes.add(Rectangle::new(18.0, 2.5)),
                    materials.add(Color::srgb(0.55, 0.4, 0.25)),
                    Vec3::ZERO,
                )],
            },
            weapon: Some(Weapon {
                name: "Wooden Stick".to_string(),
                visual: WeaponVisual {
                    mesh: meshes.add(Rectangle::new(18.0, 2.5)),
                    material: materials.add(Color::srgb(0.55, 0.4, 0.25)),
                    offset: 12.0,
                },
                damage: 1,
                speed: 2,
                reach: 2,
                arc: 2,
                impact: 1,
                attack_type: AttackType::Smash,
                damage_type: DamageType::Physical,
                rarity: Rarity::Common,
                cost: 0,
                block: 1,
                block_kb: 1,
            }),
            consumable_effect: None,
        },
    );

    // Rusty Knife
    items.insert(
        ItemId::RustyKnife,
        ItemDefinition {
            name: "Rusty Knife".to_string(),
            category: ItemCategory::Weapon,
            stack_max: 1,
            ground_visual: GroundItemVisual {
                meshes: vec![
                    (
                        meshes.add(Triangle2d::new(
                            Vec2::new(0.0, 2.0),
                            Vec2::new(0.0, -2.0),
                            Vec2::new(9.0, 0.0),
                        )),
                        materials.add(Color::srgb(0.75, 0.75, 0.8)),
                        Vec3::ZERO,
                    ),
                    (
                        meshes.add(Rectangle::new(4.0, 3.0)),
                        materials.add(Color::srgb(0.45, 0.3, 0.15)),
                        Vec3::new(-5.0, 0.0, 0.0),
                    ),
                ],
            },
            weapon: Some(Weapon {
                name: "Rusty Knife".to_string(),
                visual: WeaponVisual {
                    mesh: meshes.add(Triangle2d::new(
                        Vec2::new(0.0, 2.0),
                        Vec2::new(0.0, -2.0),
                        Vec2::new(12.0, 0.0),
                    )),
                    material: materials.add(Color::srgb(0.75, 0.75, 0.8)),
                    offset: 14.0,
                },
                damage: 1,
                speed: 3,
                reach: 3,
                arc: 1,
                impact: 3,
                attack_type: AttackType::Slash,
                damage_type: DamageType::Physical,
                rarity: Rarity::Common,
                cost: 10,
                block: 2,
                block_kb: 3,
            }),
            consumable_effect: None,
        },
    );

    // Fist
    items.insert(
        ItemId::Fist,
        ItemDefinition {
            name: "Fist".to_string(),
            category: ItemCategory::Weapon,
            stack_max: 1,
            ground_visual: GroundItemVisual {
                meshes: vec![(
                    meshes.add(Circle::new(3.0)),
                    materials.add(Color::srgb(0.8, 0.65, 0.5)),
                    Vec3::ZERO,
                )],
            },
            weapon: Some(Weapon {
                name: "Fist".to_string(),
                visual: WeaponVisual {
                    mesh: meshes.add(Circle::new(3.0)),
                    material: materials.add(Color::srgb(0.8, 0.65, 0.5)),
                    offset: 11.0,
                },
                damage: 1,
                speed: 4,
                reach: 1,
                arc: 5,
                impact: 2,
                attack_type: AttackType::Smash,
                damage_type: DamageType::Physical,
                rarity: Rarity::Common,
                cost: 0,
                block: 1,
                block_kb: 1,
            }),
            consumable_effect: None,
        },
    );

    // Health Potion
    items.insert(
        ItemId::HealthPotion,
        ItemDefinition {
            name: "Health Potion".to_string(),
            category: ItemCategory::Consumable,
            stack_max: 10,
            ground_visual: GroundItemVisual {
                meshes: vec![(
                    meshes.add(Capsule2d::new(4.0, 6.0)),
                    materials.add(Color::srgb(0.9, 0.3, 0.3)),
                    Vec3::ZERO,
                )],
            },
            weapon: None,
            consumable_effect: Some(ConsumableEffect::Heal(5)),
        },
    );

    // Leather Armor
    items.insert(
        ItemId::LeatherArmor,
        ItemDefinition {
            name: "Leather Armor".to_string(),
            category: ItemCategory::Armor,
            stack_max: 1,
            ground_visual: GroundItemVisual {
                meshes: vec![(
                    meshes.add(Rectangle::new(10.0, 12.0)),
                    materials.add(Color::srgb(0.45, 0.35, 0.25)),
                    Vec3::ZERO,
                )],
            },
            weapon: None,
            consumable_effect: None,
        },
    );

    ItemRegistry { items }
}
