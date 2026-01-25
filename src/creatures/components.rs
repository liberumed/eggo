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

/// Creature attacks snap to 4 cardinal directions (like player)
#[derive(Component)]
pub struct CardinalAttacks;

/// Creature uses sprite-based rendering (vs procedural mesh)
#[derive(Component)]
pub struct SpriteRendering;

/// Vertical offset for creature's attack origin (0.0 = feet, higher = body center)
#[derive(Component)]
pub struct AttackOffset(pub f32);

#[derive(Component)]
pub struct PatrolOrigin {
    pub position: Vec2,
}

#[derive(Component)]
pub struct AlertIndicator(pub Entity);

#[derive(Clone, Copy, Default, PartialEq)]
pub enum PatrolAction {
    #[default]
    Idle,
    Moving,
}

#[derive(Component, Default)]
pub struct PatrolWander {
    pub direction: Vec2,
    pub action: PatrolAction,
    pub action_timer: f32,
}

#[derive(Component)]
pub struct Rushing;
