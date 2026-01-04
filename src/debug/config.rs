use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct DebugConfig {
    pub show_collisions: bool,
}
