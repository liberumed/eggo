use bevy::prelude::*;

use crate::components::ItemId;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
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
                (ItemId::HealthPotion, 1, Vec2::new(30.0, 20.0)),
                (ItemId::HealthPotion, 3, Vec2::new(-40.0, 30.0)),
                (ItemId::RustyKnife, 1, Vec2::new(50.0, -10.0)),
            ],
        }
    }
}
