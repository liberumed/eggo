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
