mod attack;
mod events;
mod machine;
mod systems;
mod traits;

pub use attack::AttackPhase;
pub use events::{RequestTransition, StateEntered, StateExited};
pub use machine::StateMachine;
pub use traits::StateType;

use bevy::prelude::*;

use crate::core::GameState;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StateMachineSet {
    ProcessTransitions,
    OnEnter,
    OnExit,
    Behavior,
    Cleanup,
}

pub struct StateMachinePlugin;

impl Plugin for StateMachinePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                StateMachineSet::ProcessTransitions,
                StateMachineSet::OnEnter,
                StateMachineSet::OnExit,
                StateMachineSet::Behavior,
                StateMachineSet::Cleanup,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

pub fn register_state_type<S: StateType + Default>(app: &mut App) {
    app.add_message::<RequestTransition<S>>()
        .add_message::<StateEntered<S>>()
        .add_message::<StateExited<S>>()
        .add_systems(
            Update,
            (
                systems::process_transitions::<S>.in_set(StateMachineSet::ProcessTransitions),
                systems::tick_state_time::<S>.after(StateMachineSet::Behavior),
            )
                .run_if(in_state(GameState::Playing)),
        );
}
