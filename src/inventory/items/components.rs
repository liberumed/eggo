use bevy::prelude::*;

use super::super::Rarity;

#[allow(dead_code)]
#[derive(Component)]
pub struct Armor {
    pub name: String,
    pub defense: i32,
    pub rarity: Rarity,
}

#[allow(dead_code)]
#[derive(Component)]
pub struct Consumable {
    pub name: String,
    pub heal_amount: i32,
}
