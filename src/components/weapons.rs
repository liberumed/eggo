use bevy::prelude::*;

#[derive(Component)]
pub struct Knife;

#[derive(Component)]
pub struct Fist;

#[derive(Component)]
pub struct KnifeSwing {
    pub timer: f32,
    pub base_angle: f32,
}

#[derive(Component)]
pub struct FistSwing {
    pub timer: f32,
}
