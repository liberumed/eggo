use bevy::prelude::*;

use super::data::PropType;

#[derive(Component)]
pub struct Prop {
    pub prop_type: PropType,
}

#[derive(Component)]
pub struct Destructible {
    pub health: i32,
}

/// Component for crate sprite state
#[derive(Component)]
pub struct CrateSprite {
    pub damaged: bool,
}
