use super::super::Rarity;
use super::components::{Armor, Consumable};

/// Effect when a consumable is used
#[derive(Clone)]
pub enum ConsumableEffect {
    Heal(i32),
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
