pub mod data;

pub use data::*;

use bevy::prelude::*;

use crate::resources::GameState;
use crate::systems::{
    animate_creatures, animate_death, apply_collision_push,
};
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
