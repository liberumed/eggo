use bevy::prelude::*;

#[derive(Component)]
pub struct Health(pub i32);

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct Stunned(pub f32);

#[derive(Component)]
pub struct Knockback {
    pub velocity: Vec2,
    pub timer: f32,
}

#[derive(Component)]
pub struct Blocking;

#[derive(Component)]
pub struct DespawnTimer(pub f32);

/// Shadow component for characters
#[derive(Component)]
pub struct Shadow;

#[derive(Component)]
pub struct DeathAnimation {
    pub timer: f32,
    pub stage: u8,
}

#[derive(Component, Clone, Copy)]
pub struct Loot {
    pub philosophy: bool,
    pub nature_study: bool,
    pub wisdom: bool,
}
