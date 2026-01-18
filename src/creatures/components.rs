use bevy::prelude::*;

use super::data::SteeringConfig;

/// Creature marker component
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

/// Marker for creatures that became hostile after being hit (angry, direct pursuit)
/// Creatures without this use flanking behavior
#[derive(Component)]
pub struct Provoked;

/// Active steering configuration for a creature
#[derive(Component, Clone, Debug)]
pub struct CreatureSteering(pub SteeringConfig);

/// Stored steering config to use when creature becomes provoked
#[derive(Component, Clone, Debug)]
pub struct ProvokedSteering(pub SteeringConfig);

/// Marker for Goblin enemies (use player sprites, half-circle attacks)
#[derive(Component)]
pub struct Goblin;

/// Marker for creatures that have spotted the player and are actively pursuing
#[derive(Component)]
pub struct Activated;
