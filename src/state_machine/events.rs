use bevy::prelude::*;

use super::StateType;

#[derive(Event, Message)]
pub struct RequestTransition<S: StateType> {
    pub entity: Entity,
    pub target: S,
    pub force: bool,
}

impl<S: StateType> RequestTransition<S> {
    pub fn new(entity: Entity, target: S) -> Self {
        Self {
            entity,
            target,
            force: false,
        }
    }

    #[allow(dead_code)]
    pub fn forced(entity: Entity, target: S) -> Self {
        Self {
            entity,
            target,
            force: true,
        }
    }
}

#[derive(Event, Message)]
pub struct StateEntered<S: StateType> {
    pub entity: Entity,
    pub state: S,
}

#[derive(Event, Message)]
pub struct StateExited<S: StateType> {
    pub entity: Entity,
    pub state: S,
}
