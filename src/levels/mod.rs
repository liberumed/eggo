mod data;
mod spawner;
mod systems;

pub use data::{CurrentLevel, EntityType, LevelData};
pub use spawner::{spawn_level_background, LevelBackground, VoidBackground};
pub use systems::BoundToLevel;

use bevy::prelude::*;

use crate::core::GameState;

pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
            .add_systems(
                PostUpdate,
                systems::enforce_level_bounds.run_if(in_state(GameState::Playing)),
            );
    }
}
