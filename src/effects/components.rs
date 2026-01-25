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

/// Dust particle from sprinting
#[derive(Component)]
pub struct DustParticle {
    pub lifetime: f32,
}

/// Red flash highlight when enemy is hit
#[derive(Component)]
pub struct HitHighlight {
    pub timer: f32,
    pub duration: f32,
    pub original_material: Option<Handle<ColorMaterial>>,
}

/// Floating damage number that rises and fades
#[derive(Component)]
pub struct DamageNumber {
    pub velocity: Vec2,
    pub lifetime: f32,
}
