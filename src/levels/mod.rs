mod data;
mod spawner;
pub mod systems;

pub use data::{CurrentLevel, EntityType, LevelData};
pub use spawner::{spawn_level_background, spawn_win_zone, LevelBackground, VoidBackground, WinZone, WinZoneTimer, WinZoneTimerText};
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
