use bevy::prelude::*;

use super::AttackType;

/// Marker component for knife weapons
#[derive(Component)]
pub struct Knife;

/// Marker component for stick weapons
#[derive(Component)]
pub struct Stick;

/// Marker component for fist weapons
#[derive(Component)]
pub struct Fist;

/// Marker for weapon visual mesh children - used to find and despawn when swapping
#[derive(Component)]
pub struct WeaponVisualMesh;

/// Active weapon swing state
#[derive(Component)]
pub struct WeaponSwing {
    pub timer: f32,
    pub duration: f32,
    pub base_angle: Option<f32>,
    pub attack_type: AttackType,
    pub hit_delay: f32,
    pub hit_applied: bool,
}

/// Marker for a weapon that is currently drawn/visible
#[derive(Component)]
pub struct Drawn;

/// Marker for the player's weapon entity
#[derive(Component)]
pub struct PlayerWeapon;
