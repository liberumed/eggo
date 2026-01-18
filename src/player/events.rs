use bevy::prelude::*;

use super::FacingDirection;

#[derive(Event, Message, Debug)]
pub struct DashInputDetected {
    pub player: Entity,
    pub direction: Vec2,
}

#[derive(Event, Message, Debug)]
pub struct AttackInputDetected {
    pub player: Entity,
    pub facing_direction: FacingDirection,
    pub attack_angle: f32,
}

#[derive(Event, Message, Debug)]
pub struct MovementInputDetected {
    pub player: Entity,
    pub direction: Vec2,
}
