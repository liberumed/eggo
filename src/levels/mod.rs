mod data;
mod spawner;
pub mod systems;

pub use data::{CreatureType, CurrentLevel, LevelData, PitData, PropType};
pub use spawner::{spawn_level_background, spawn_pit, spawn_win_zone, LevelBackground, Pit, VoidBackground, WinZone, WinZoneTimer, WinZoneTimerText};
pub use systems::{BoundToLevel, FallingIntoPit, WaveSpawnState};

use bevy::prelude::*;

use crate::core::GameState;

pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
            .add_systems(
                Update,
                (
                    systems::apply_pit_edge_resistance,
                    systems::check_pit_fall.after(systems::apply_pit_edge_resistance),
                    systems::animate_pit_fall.after(systems::check_pit_fall),
                ).run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                systems::enforce_level_bounds.run_if(in_state(GameState::Playing)),
            );
    }
}
