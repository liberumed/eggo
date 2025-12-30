use bevy::prelude::*;

#[derive(Component)]
pub struct Knife;

#[derive(Component)]
pub struct Fist;

#[derive(Component)]
pub struct WeaponSwing {
    pub timer: f32,
    pub duration: f32,
    pub base_angle: Option<f32>,
}
