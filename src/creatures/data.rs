#![allow(dead_code)]

/// Steering behavior strategy
#[derive(Clone, Copy, Default, Debug)]
pub enum SteeringStrategy {
    #[default]
    Direct,   // seek_interest - straight toward player
    Flanking, // seek_with_flank - approach from angles
}

/// Configuration for creature steering AI behavior
#[derive(Clone, Debug)]
pub struct SteeringConfig {
    pub strategy: SteeringStrategy,
    pub sight_range: f32,           // How far can see player
    pub obstacle_look_ahead: f32,   // Obstacle avoidance distance
    pub separation_radius: f32,     // Distance from other creatures
    pub min_player_distance: f32,   // Don't get closer than this
    pub flank_angle_min: f32,       // Min flank angle (radians)
    pub flank_angle_max: f32,       // Max flank angle (radians)
    pub occupied_angle_spread: f32, // Angle spread for occupied_angle_danger
}

impl Default for SteeringConfig {
    fn default() -> Self {
        Self {
            strategy: SteeringStrategy::Direct,
            sight_range: 200.0,
            obstacle_look_ahead: 50.0,
            separation_radius: 35.0,
            min_player_distance: 25.0,
            flank_angle_min: 0.5,       // ~30°
            flank_angle_max: 1.0,       // ~60°
            occupied_angle_spread: 0.6, // ~35°
        }
    }
}

/// Identifier for creature types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreatureId {
    Blob,
    Goblin,
}

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
    pub hostile_chance: f64,
    pub glowing_chance: f64,
    pub loot: LootTable,
    // Physical properties
    pub walk_collider: ColliderDef,
    pub hit_collider: ColliderDef,
    pub base_offset: f32,
    pub scale: f32,
    // AI behavior
    pub steering: SteeringConfig,
    pub provoked_steering: SteeringConfig,
}

pub mod creature_catalog {
    use super::*;

    /// Neutral blob - can become hostile when provoked
    pub fn blob() -> CreatureDefinition {
        CreatureDefinition {
            name: "Blob".to_string(),
            health: 2,
            speed: 55.0,
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
            // Neutral blobs don't have steering (not hostile by default)
            steering: SteeringConfig::default(),
            provoked_steering: SteeringConfig {
                strategy: SteeringStrategy::Direct,
                sight_range: 300.0, // Longer range when angry
                ..Default::default()
            },
        }
    }

    /// Hostile blob - always aggressive, tougher
    pub fn hostile_blob() -> CreatureDefinition {
        CreatureDefinition {
            name: "Hostile Blob".to_string(),
            health: 6,
            speed: 55.0,
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
            // Hostile blobs use flanking behavior
            steering: SteeringConfig {
                strategy: SteeringStrategy::Flanking,
                sight_range: 200.0,
                flank_angle_min: 0.5,
                flank_angle_max: 1.0,
                ..Default::default()
            },
            // Not used (always hostile from start)
            provoked_steering: SteeringConfig::default(),
        }
    }

    /// Goblin - humanoid enemy with club, uses player-like sprites
    pub fn goblin() -> CreatureDefinition {
        CreatureDefinition {
            name: "Goblin".to_string(),
            health: 4,
            speed: 50.0,
            hostile_chance: 1.0,  // Always hostile
            glowing_chance: 0.0,
            loot: LootTable {
                philosophy_chance: 0.3,
                nature_chance: 0.3,
                wisdom_chance: 0.4,
            },
            // Same colliders as player for sprite-based enemy
            walk_collider: ColliderDef::new(8.0, 4.0, -4.0),
            hit_collider: ColliderDef::new(10.0, 14.0, 5.0),
            base_offset: -24.0,  // Same as player for 64x64 sprites
            scale: 1.0,
            // Goblins use flanking behavior
            steering: SteeringConfig {
                strategy: SteeringStrategy::Flanking,
                sight_range: 200.0,
                flank_angle_min: 0.3,
                flank_angle_max: 0.8,
                ..Default::default()
            },
            provoked_steering: SteeringConfig::default(),
        }
    }
}
