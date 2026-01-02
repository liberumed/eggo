use bevy::prelude::*;

/// Creature marker component (creatures in the world)
#[derive(Component)]
pub struct Creature;

#[derive(Component)]
pub struct CreatureAnimation {
    pub phase: f32,
    pub speed: f32,
    pub amplitude: f32,
}

/// Marker for creatures that will attack the player
#[derive(Component)]
pub struct Hostile {
    pub speed: f32,
}

/// Marker for glowing creatures
#[derive(Component)]
pub struct Glowing;

/// Identifier for creature types (for future creature variety)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreatureId {
    Blob,
}

/// Loot drop chances for a creature
#[derive(Clone)]
pub struct LootTable {
    pub philosophy_chance: f64,
    pub nature_chance: f64,
    pub wisdom_chance: f64,
}

/// Complete definition of a creature type
#[allow(dead_code)]
#[derive(Clone)]
pub struct CreatureDefinition {
    pub name: String,
    pub health: i32,
    pub speed: f32,
    pub damage: i32,
    pub hostile_chance: f64,
    pub glowing_chance: f64,
    pub loot: LootTable,
}

pub mod creature_catalog {
    use super::*;

    /// Neutral blob - can become hostile when provoked
    pub fn blob() -> CreatureDefinition {
        CreatureDefinition {
            name: "Blob".to_string(),
            health: 2,
            speed: 55.0,
            damage: 1,
            hostile_chance: 0.0,
            glowing_chance: 0.35,
            loot: LootTable {
                philosophy_chance: 0.5,
                nature_chance: 0.5,
                wisdom_chance: 0.5,
            },
        }
    }

    /// Hostile blob - always aggressive, tougher
    pub fn hostile_blob() -> CreatureDefinition {
        CreatureDefinition {
            name: "Hostile Blob".to_string(),
            health: 6,
            speed: 55.0,
            damage: 1,
            hostile_chance: 1.0,
            glowing_chance: 0.0,
            loot: LootTable {
                philosophy_chance: 0.6,
                nature_chance: 0.6,
                wisdom_chance: 0.6,
            },
        }
    }
}
