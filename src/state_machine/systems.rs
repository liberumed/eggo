use bevy::prelude::*;

use super::{RequestTransition, StateEntered, StateExited, StateMachine, StateType};

pub fn process_transitions<S: StateType>(
    mut requests: MessageReader<RequestTransition<S>>,
    mut entered: MessageWriter<StateEntered<S>>,
    mut exited: MessageWriter<StateExited<S>>,
    mut query: Query<&mut StateMachine<S>>,
) {
    for request in requests.read() {
        let Ok(mut state_machine) = query.get_mut(request.entity) else {
            continue;
        };

        if !request.force && !state_machine.current().can_transition_to(&request.target) {
            continue;
        }

        if state_machine.current() == &request.target {
            continue;
        }

        let previous = state_machine.current().clone();

        exited.write(StateExited {
            entity: request.entity,
            state: previous,
        });

        state_machine.transition_to(request.target.clone());

        entered.write(StateEntered {
            entity: request.entity,
            state: request.target.clone(),
        });
    }
}

pub fn tick_state_time<S: StateType>(time: Res<Time>, mut query: Query<&mut StateMachine<S>>) {
    for mut state_machine in &mut query {
        state_machine.tick(time.delta_secs());
    }
}
