use bevy::prelude::*;

/// Emitted when a creature detects the player within weapon range
#[derive(Event, Message, Debug)]
pub struct PlayerInRange {
    pub creature: Entity,
    pub distance: f32,
}
