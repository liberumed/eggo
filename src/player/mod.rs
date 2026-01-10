pub mod components;
pub mod events;
pub mod spawner;
pub mod sprites;
pub mod state;
pub mod state_handlers;
pub mod stats;
pub mod systems;

pub use components::*;
pub use events::*;
pub use spawner::*;
pub use sprites::*;
pub use state::*;
pub use state_handlers::*;
pub use stats::*;
pub use systems::*;

use bevy::prelude::*;

use crate::core::GameState;
use crate::combat::{
    aim_weapon, toggle_weapon, sync_range_indicator, update_weapon_visual,
    handle_block, apply_mesh_attack_hits, apply_smash_attack_hits,
};
use crate::inventory::cursor_not_over_ui;
use crate::state_machine::{register_state_type, StateMachineSet};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        register_state_type::<PlayerState>(app);

        app.add_message::<DashInputDetected>()
            .add_message::<AttackInputDetected>()
            .add_message::<MovementInputDetected>();

        app.add_systems(
            Update,
            (
                detect_movement_input.in_set(StateMachineSet::Behavior),
                detect_dash_input.in_set(StateMachineSet::Behavior),
                detect_attack_input.in_set(StateMachineSet::Behavior),
                handle_movement_input.in_set(StateMachineSet::Behavior)
                    .after(detect_movement_input),
                handle_dash_input.in_set(StateMachineSet::Behavior)
                    .after(detect_dash_input),
                handle_attack_input.in_set(StateMachineSet::Behavior)
                    .after(detect_attack_input),
            )
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(
            Update,
            (
                on_dashing_exit.in_set(StateMachineSet::OnExit),
                on_attacking_windup_enter.in_set(StateMachineSet::OnEnter),
                on_attacking_exit.in_set(StateMachineSet::OnExit),
                on_idle_enter.in_set(StateMachineSet::OnEnter),
            )
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(
            Update,
            (
                move_player,
                apply_dash_state,
                tick_dash_cooldown,
                tick_phase_through,
                apply_knockback,
                animate_player_death,
                toggle_weapon,
                aim_weapon,
                sync_range_indicator,
                update_weapon_visual,
                advance_player_attack_phases,
                apply_mesh_attack_hits,
                apply_smash_attack_hits,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            animate_weapon_swing
                .in_set(StateMachineSet::Cleanup)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            handle_block
                .run_if(in_state(GameState::Playing))
                .run_if(cursor_not_over_ui),
        )
        .add_systems(Update, camera_follow);
    }
}
