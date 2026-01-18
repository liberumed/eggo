#![allow(dead_code)]

use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use super::items::ConsumableEffect;
use super::weapons::{Weapon, weapon_catalog};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum ItemId {
    WoodenStick,
    RustyKnife,
    Sword,
    Fist,
    HealthPotion,
    Mushroom,
    LeatherArmor,
}

/// Resource holding item icon textures for UI and ground display
#[derive(Resource, Default)]
pub struct ItemIcons {
    pub icons: HashMap<ItemId, Handle<Image>>,
    pub ground_icons: HashMap<ItemId, Handle<Image>>,
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
    pub meshes: Vec<(Handle<Mesh>, Handle<ColorMaterial>, Vec3)>,
}

/// Complete definition of an item
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

/// Simple item data for backward compatibility
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
        ItemId::Sword => ItemData {
            id,
            name: "Sword".to_string(),
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
        ItemId::Mushroom => ItemData {
            id,
            name: "Mushroom".to_string(),
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
) -> Option<Weapon> {
    match id {
        ItemId::WoodenStick => Some(weapon_catalog::wooden_stick(meshes, materials)),
        ItemId::RustyKnife => Some(weapon_catalog::rusty_knife(meshes, materials)),
        ItemId::Sword => Some(weapon_catalog::sword(meshes, materials)),
        ItemId::Fist => Some(weapon_catalog::fist(meshes, materials)),
        _ => None,
    }
}

/// Builds the item registry with all item definitions
pub fn build_item_registry(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> ItemRegistry {
    let mut items = HashMap::new();

    // Wooden Stick
    let stick_weapon = weapon_catalog::wooden_stick(meshes, materials);
    items.insert(
        ItemId::WoodenStick,
        ItemDefinition {
            name: stick_weapon.name.clone(),
            category: ItemCategory::Weapon,
            stack_max: 1,
            ground_visual: GroundItemVisual {
                meshes: vec![(
                    stick_weapon.visual.mesh.clone(),
                    stick_weapon.visual.material.clone(),
                    Vec3::ZERO,
                )],
            },
            weapon: Some(stick_weapon),
            consumable_effect: None,
        },
    );

    // Rusty Knife
    let knife_weapon = weapon_catalog::rusty_knife(meshes, materials);
    items.insert(
        ItemId::RustyKnife,
        ItemDefinition {
            name: knife_weapon.name.clone(),
            category: ItemCategory::Weapon,
            stack_max: 1,
            ground_visual: GroundItemVisual {
                meshes: vec![
                    (
                        knife_weapon.visual.mesh.clone(),
                        knife_weapon.visual.material.clone(),
                        Vec3::ZERO,
                    ),
                    (
                        meshes.add(Rectangle::new(4.0, 3.0)),
                        materials.add(Color::srgb(0.45, 0.3, 0.15)),
                        Vec3::new(-5.0, 0.0, 0.0),
                    ),
                ],
            },
            weapon: Some(knife_weapon),
            consumable_effect: None,
        },
    );

    // Sword
    let sword_weapon = weapon_catalog::sword(meshes, materials);
    items.insert(
        ItemId::Sword,
        ItemDefinition {
            name: sword_weapon.name.clone(),
            category: ItemCategory::Weapon,
            stack_max: 1,
            ground_visual: GroundItemVisual {
                meshes: vec![
                    (
                        sword_weapon.visual.mesh.clone(),
                        sword_weapon.visual.material.clone(),
                        Vec3::ZERO,
                    ),
                    (
                        meshes.add(Rectangle::new(3.0, 5.0)),
                        materials.add(Color::srgb(0.4, 0.3, 0.2)),
                        Vec3::new(-12.0, 0.0, 0.0),
                    ),
                ],
            },
            weapon: Some(sword_weapon),
            consumable_effect: None,
        },
    );

    // Fist
    let fist_weapon = weapon_catalog::fist(meshes, materials);
    items.insert(
        ItemId::Fist,
        ItemDefinition {
            name: fist_weapon.name.clone(),
            category: ItemCategory::Weapon,
            stack_max: 1,
            ground_visual: GroundItemVisual {
                meshes: vec![(
                    fist_weapon.visual.mesh.clone(),
                    fist_weapon.visual.material.clone(),
                    Vec3::ZERO,
                )],
            },
            weapon: Some(fist_weapon),
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

    // Mushroom
    items.insert(
        ItemId::Mushroom,
        ItemDefinition {
            name: "Mushroom".to_string(),
            category: ItemCategory::Consumable,
            stack_max: 10,
            ground_visual: GroundItemVisual {
                meshes: vec![],  // Uses sprite icon instead
            },
            weapon: None,
            consumable_effect: Some(ConsumableEffect::Heal(3)),
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
