pub mod data;
pub mod spawner;
pub mod steering;
pub mod systems;

pub use data::*;
pub use spawner::*;
pub use steering::*;
pub use systems::*;

use bevy::prelude::*;

use crate::core::GameState;
use crate::combat::{
    apply_creature_delayed_hits, hostile_ai, hostile_attack,
    hostile_fist_aim, sync_creature_range_indicators,
};

pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                hostile_ai,
                hostile_fist_aim,
                hostile_attack,
                apply_creature_delayed_hits,
                apply_collision_push,
                animate_creatures,
                animate_death,
                sync_creature_range_indicators,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
