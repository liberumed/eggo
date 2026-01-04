use bevy::prelude::*;

/// Marker for debug collision circle (walk collision)
#[derive(Component)]
pub struct CollisionDebugCircle;

/// Marker for debug hit collision circle (hurtbox)
#[derive(Component)]
pub struct HitDebugCircle;

/// Marker for debug weapon reach cone
#[derive(Component)]
pub struct WeaponReachCone;

/// Marker for entities that have debug circles spawned
#[derive(Component)]
pub struct HasDebugCircle;

/// Marker for weapons that have debug cone spawned
#[derive(Component)]
pub struct HasDebugCone;

/// Stores current weapon stats for the debug cone (to detect changes)
#[derive(Component)]
pub struct WeaponConeStats {
    pub range: f32,
    pub half_angle: f32,
}

/// Links a debug circle to its owner creature
#[derive(Component)]
pub struct CreatureDebugCircle(pub Entity);
