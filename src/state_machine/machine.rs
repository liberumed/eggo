use bevy::prelude::*;

use super::StateType;

#[derive(Component, Clone, Debug)]
pub struct StateMachine<S: StateType> {
    current: S,
    previous: Option<S>,
    time_in_state: f32,
}

impl<S: StateType + Default> Default for StateMachine<S> {
    fn default() -> Self {
        Self {
            current: S::default(),
            previous: None,
            time_in_state: 0.0,
        }
    }
}

impl<S: StateType> StateMachine<S> {
    pub fn new(initial: S) -> Self {
        Self {
            current: initial,
            previous: None,
            time_in_state: 0.0,
        }
    }

    pub fn current(&self) -> &S {
        &self.current
    }

    #[allow(dead_code)]
    pub fn previous(&self) -> Option<&S> {
        self.previous.as_ref()
    }

    pub fn time_in_state(&self) -> f32 {
        self.time_in_state
    }

    #[allow(dead_code)]
    pub fn is(&self, state: &S) -> bool {
        &self.current == state
    }

    pub(crate) fn transition_to(&mut self, target: S) {
        self.previous = Some(std::mem::replace(&mut self.current, target));
        self.time_in_state = 0.0;
    }

    pub(crate) fn tick(&mut self, delta: f32) {
        self.time_in_state += delta;
    }
}
