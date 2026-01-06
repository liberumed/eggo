use bevy::prelude::*;

#[derive(Event, Message, Debug)]
pub struct DashInputDetected {
    pub player: Entity,
    pub direction: Vec2,
}

#[derive(Event, Message, Debug)]
pub struct AttackInputDetected {
    pub player: Entity,
    pub facing_right: bool,
}

#[derive(Event, Message, Debug)]
pub struct MovementInputDetected {
    pub player: Entity,
    pub direction: Vec2,
}
