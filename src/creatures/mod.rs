pub mod components;
pub mod data;
pub mod events;
pub mod spawner;
pub mod state;
pub mod state_handlers;
pub mod steering;
pub mod systems;

pub use components::*;
pub use data::*;
pub use events::*;
pub use spawner::*;
pub use state::*;
pub use state_handlers::*;
pub use steering::*;
pub use systems::*;

use bevy::prelude::*;

use crate::core::GameState;
use crate::combat::{
    alert_ai, process_creature_attacks, hostile_ai, hostile_attack,
    hostile_fist_aim, patrol_ai, sync_creature_range_indicators, update_goblin_attack_indicator,
};
use crate::state_machine::{register_state_type, StateMachineSet};

pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        register_state_type::<CreatureState>(app);

        // Register creature events
        app.add_message::<PlayerInRange>();

        app.add_systems(
            Update,
            (
                on_attack_windup_enter.in_set(StateMachineSet::OnEnter),
                on_attack_exit.in_set(StateMachineSet::OnExit),
                on_creature_provoked.in_set(StateMachineSet::OnEnter),
                on_creature_stunned.in_set(StateMachineSet::OnEnter),
                on_stun_recovered.in_set(StateMachineSet::OnExit),
                on_hostile_start_patrol.in_set(StateMachineSet::OnEnter),
                on_activated_to_chase.in_set(StateMachineSet::OnEnter),
                on_alert_enter.in_set(StateMachineSet::OnEnter),
                on_alert_exit.in_set(StateMachineSet::OnExit),
                on_deactivated_to_patrol.in_set(StateMachineSet::OnExit),
            )
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(
            Update,
            (
                detect_player_proximity.in_set(StateMachineSet::Behavior),
                patrol_ai.in_set(StateMachineSet::Behavior),
                alert_ai.in_set(StateMachineSet::Behavior),
                hostile_ai.in_set(StateMachineSet::Behavior),
                hostile_fist_aim.in_set(StateMachineSet::Behavior),
                hostile_attack.in_set(StateMachineSet::Behavior).after(detect_player_proximity),
                advance_attack_phases.in_set(StateMachineSet::Behavior),
                advance_cooldown_state.in_set(StateMachineSet::Behavior),
                process_creature_attacks.in_set(StateMachineSet::Behavior),
            )
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(
            Update,
            (
                apply_collision_push,
                animate_creatures,
                animate_death,
                sync_creature_range_indicators,
                update_goblin_attack_indicator,
                update_goblin_sprite_animation,
                update_alert_indicator,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
