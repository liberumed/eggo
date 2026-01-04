use bevy::prelude::*;

use crate::components::ItemId;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
    Dead,
}

#[derive(Resource)]
pub struct WorldConfig {
    pub starting_items: Vec<(ItemId, u32, Vec2)>,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            starting_items: vec![
                (ItemId::Mushroom, 1, Vec2::new(30.0, 20.0)),
                (ItemId::Mushroom, 3, Vec2::new(-40.0, 30.0)),
                (ItemId::RustyKnife, 1, Vec2::new(50.0, -10.0)),
            ],
        }
    }
}

/// Flag to indicate new game was requested (needs cleanup before spawn)
#[derive(Resource, Default)]
pub struct NewGameRequested(pub bool);
