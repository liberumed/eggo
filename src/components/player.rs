use bevy::prelude::*;

/// Player marker component
#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct PlayerAnimation {
    pub time: f32,
    pub velocity: Vec2,
}
