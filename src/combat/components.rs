use bevy::prelude::*;

use super::weapons::AttackType;

#[derive(Component)]
pub struct Knife;

#[derive(Component)]
pub struct Stick;

#[derive(Component)]
pub struct Fist;

#[derive(Component)]
pub struct PlayerWeapon;

#[derive(Component)]
pub struct WeaponSwing {
    pub timer: f32,
    pub duration: f32,
    pub base_angle: Option<f32>,
    pub attack_type: AttackType,
    pub hit_delay: f32,
    pub hit_applied: bool,
}

/// Marker for weapon visual mesh children
#[derive(Component)]
pub struct WeaponVisualMesh;

/// Marker for weapon range indicator
#[derive(Component)]
pub struct WeaponRangeIndicator;

/// Marker for player's range indicator
#[derive(Component)]
pub struct PlayerRangeIndicator;

/// Links a creature's range indicator to its owner
#[derive(Component)]
pub struct CreatureRangeIndicator(pub Entity);

/// Marker for drawn weapon
#[derive(Component)]
pub struct Drawn;
