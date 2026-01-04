/// Loot drop chances for a creature
#[derive(Clone)]
pub struct LootTable {
    pub philosophy_chance: f64,
    pub nature_chance: f64,
    pub wisdom_chance: f64,
}

/// Collider dimensions (radius_x, radius_y, offset_y)
#[derive(Clone, Copy)]
pub struct ColliderDef {
    pub radius_x: f32,
    pub radius_y: f32,
    pub offset_y: f32,
}

impl ColliderDef {
    pub const fn new(radius_x: f32, radius_y: f32, offset_y: f32) -> Self {
        Self { radius_x, radius_y, offset_y }
    }
}

/// Complete definition of a creature type
#[derive(Clone)]
pub struct CreatureDefinition {
    pub name: String,
    pub health: i32,
    pub speed: f32,
    pub damage: i32,
    pub hostile_chance: f64,
    pub glowing_chance: f64,
    pub loot: LootTable,
    // Physical properties
    pub walk_collider: ColliderDef,
    pub hit_collider: ColliderDef,
    pub base_offset: f32,
    pub scale: f32,
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
            walk_collider: ColliderDef::new(8.0, 5.0, -11.0),
            hit_collider: ColliderDef::new(10.0, 14.0, 0.0),
            base_offset: -14.0,
            scale: 1.0,
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
            walk_collider: ColliderDef::new(8.0, 5.0, -11.0),
            hit_collider: ColliderDef::new(10.0, 14.0, 0.0),
            base_offset: -14.0,
            scale: 1.0,
        }
    }
}
