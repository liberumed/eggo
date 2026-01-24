use crate::state_machine::{AttackPhase, StateType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CreatureState {
    #[default]
    Idle,
    Patrol,
    Chase,
    Attack(AttackPhase),
    /// Post-attack cooldown - creature waits before attacking again
    Cooldown,
    #[allow(dead_code)]
    Stunned,
    #[allow(dead_code)]
    Dying,
    #[allow(dead_code)]
    Dead,
}

impl StateType for CreatureState {
    fn can_transition_to(&self, target: &Self) -> bool {
        use CreatureState::*;

        match (self, target) {
            // Dead is terminal
            (Dead, _) => false,

            // Dying can only go to Dead
            (Dying, Dead) => true,
            (Dying, _) => false,

            // Stunned can recover to Idle, Patrol, or Chase
            (Stunned, Idle) => true,
            (Stunned, Patrol) => true,
            (Stunned, Chase) => true,
            (Stunned, Dying) => true,
            (Stunned, Dead) => true,
            (Stunned, _) => false,

            // Anyone can die, start dying, or get stunned
            (_, Dead) => true,
            (_, Dying) => true,
            (_, Stunned) => true,

            // Idle can start patrolling or chasing
            (Idle, Patrol) => true,
            (Idle, Chase) => true,
            (Idle, _) => false,

            // Patrol can chase or return to idle
            (Patrol, Chase) => true,
            (Patrol, Idle) => true,
            (Patrol, _) => false,

            // Chase can attack, patrol, or return to idle
            (Chase, Idle) => true,
            (Chase, Patrol) => true,
            (Chase, Attack(_)) => true,
            (Chase, _) => false,

            // Attack transitions to Cooldown after recovery (or between phases)
            (Attack(_), Attack(_)) => true,
            (Attack(_), Cooldown) => true,
            (Attack(_), Chase) => true,
            (Attack(_), Patrol) => true,
            (Attack(_), Idle) => true,

            // Cooldown returns to Chase (or Idle)
            (Cooldown, Chase) => true,
            (Cooldown, Idle) => true,
            (Cooldown, _) => false,
        }
    }
}
