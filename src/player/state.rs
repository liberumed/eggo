use crate::state_machine::{AttackPhase, StateType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlayerState {
    #[default]
    Idle,
    Moving,
    Dashing,
    Attacking(AttackPhase),
    #[allow(dead_code)]
    Stunned,
    #[allow(dead_code)]
    Dying,
    #[allow(dead_code)]
    Dead,
}

impl StateType for PlayerState {
    fn can_transition_to(&self, target: &Self) -> bool {
        use PlayerState::*;

        match (self, target) {
            // Dead is terminal
            (Dead, _) => false,

            // Dying can only go to Dead
            (Dying, Dead) => true,
            (Dying, _) => false,

            // Stunned can recover to Idle
            (Stunned, Idle) => true,
            (Stunned, Dying) => true,
            (Stunned, Dead) => true,
            (Stunned, _) => false,

            // Anyone can die, start dying, or get stunned
            (_, Dead) => true,
            (_, Dying) => true,
            (_, Stunned) => true,

            // Idle ↔ Moving
            (Idle, Moving) => true,
            (Moving, Idle) => true,

            // Idle/Moving can dash or attack
            (Idle, Dashing) => true,
            (Moving, Dashing) => true,
            (Idle, Attacking(_)) => true,
            (Moving, Attacking(_)) => true,

            // Dashing returns to Moving (with momentum)
            (Dashing, Moving) => true,
            (Dashing, _) => false,

            // Attack can transition between phases or return to Idle
            (Attacking(_), Attacking(_)) => true,
            (Attacking(_), Idle) => true,
            (Attacking(_), _) => false,

            // Anything else is not allowed
            (Idle, _) => false,
            (Moving, _) => false,
        }
    }
}
