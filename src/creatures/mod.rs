pub mod data;
pub mod spawner;
pub mod state;
pub mod state_handlers;
pub mod steering;
pub mod systems;

pub use data::*;
pub use spawner::*;
pub use state::*;
pub use state_handlers::*;
pub use steering::*;
pub use systems::*;

use bevy::prelude::*;

use crate::core::GameState;
use crate::combat::{
    process_creature_attacks, hostile_ai, hostile_attack,
    hostile_fist_aim, sync_creature_range_indicators,
};
use crate::state_machine::{register_state_type, StateMachineSet};

pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        register_state_type::<CreatureState>(app);

        app.add_systems(
            Update,
            (
                // State enter/exit handlers
                on_attack_windup_enter.in_set(StateMachineSet::OnEnter),
                on_attack_exit.in_set(StateMachineSet::OnExit),
                on_creature_provoked.in_set(StateMachineSet::OnEnter),
                // Behavior systems
                hostile_ai.in_set(StateMachineSet::Behavior),
                hostile_fist_aim.in_set(StateMachineSet::Behavior),
                hostile_attack.in_set(StateMachineSet::Behavior),
                advance_attack_phases.in_set(StateMachineSet::Behavior),
                process_creature_attacks.in_set(StateMachineSet::Behavior),
                // Other systems
                apply_collision_push,
                animate_creatures,
                animate_death,
                sync_creature_range_indicators,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
