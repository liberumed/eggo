use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
    Dead,
}
