use bevy::prelude::*;

#[derive(Component)]
pub struct BloodParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
}

#[derive(Component)]
pub struct ResourceBall {
    pub velocity: Vec2,
}

#[derive(Component)]
pub struct MagnetizedBall;

/// Outline showing target direction
#[derive(Component)]
pub struct TargetOutline;
