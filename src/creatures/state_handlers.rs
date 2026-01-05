use bevy::prelude::*;

use crate::combat::{Fist, Weapon, WeaponSwing};
use crate::constants::ATTACK_HIT_DELAY_PERCENT;
use crate::core::{Dead, DeathAnimation, HitCollider, Stunned};
use crate::player::Player;
use crate::state_machine::{AttackPhase, RequestTransition, StateEntered, StateExited, StateMachine};
use super::{Creature, CreatureState, Hostile, PlayerInRange};

pub fn on_attack_windup_enter(
    mut commands: Commands,
    mut events: MessageReader<StateEntered<CreatureState>>,
    player_query: Query<&Transform, With<Player>>,
    creature_query: Query<(&Transform, &Children)>,
    fist_query: Query<(Entity, &Weapon), With<Fist>>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();

    for event in events.read() {
        if !matches!(event.state, CreatureState::Attack(AttackPhase::WindUp)) {
            continue;
        }

        let Ok((creature_transform, children)) = creature_query.get(event.entity) else {
            continue;
        };

        let creature_pos = creature_transform.translation.truncate();
        let dir = player_pos - creature_pos;
        let base_angle = Some(dir.y.atan2(dir.x));

        for child in children.iter() {
            if let Ok((fist_entity, weapon)) = fist_query.get(child) {
                let duration = weapon.swing_duration();
                commands.entity(fist_entity).insert(WeaponSwing {
                    timer: 0.0,
                    duration,
                    base_angle,
                    attack_type: weapon.attack_type,
                    hit_delay: duration * ATTACK_HIT_DELAY_PERCENT,
                    hit_applied: false,
                });
                break;
            }
        }
    }
}

pub fn on_attack_exit(
    mut commands: Commands,
    mut events: MessageReader<StateExited<CreatureState>>,
    creature_query: Query<(&Children, &StateMachine<CreatureState>)>,
    fist_query: Query<Entity, (With<Fist>, With<WeaponSwing>)>,
) {
    for event in events.read() {
        // Only handle exits from Attack states
        if !matches!(event.state, CreatureState::Attack(_)) {
            continue;
        }

        let Ok((children, state_machine)) = creature_query.get(event.entity) else {
            continue;
        };

        // Only remove WeaponSwing if we're no longer in ANY Attack state
        // (skip during phase transitions like WindUp → Strike → Recovery)
        if matches!(state_machine.current(), CreatureState::Attack(_)) {
            continue;
        }

        for child in children.iter() {
            if fist_query.get(child).is_ok() {
                commands.entity(child).remove::<WeaponSwing>();
            }
        }
    }
}

/// Transitions creatures from Idle to Chase when they become hostile (provoked)
pub fn on_creature_provoked(
    mut transitions: MessageWriter<RequestTransition<CreatureState>>,
    query: Query<(Entity, &StateMachine<CreatureState>), Added<Hostile>>,
) {
    for (entity, state_machine) in &query {
        // Only transition if currently Idle
        if *state_machine.current() == CreatureState::Idle {
            transitions.write(RequestTransition::new(entity, CreatureState::Chase));
        }
    }
}

/// Detects when creatures in Chase state are within weapon range of the player.
/// Emits PlayerInRange event for other systems to react to.
pub fn detect_player_proximity(
    mut events: MessageWriter<PlayerInRange>,
    player_query: Query<(&Transform, Option<&HitCollider>), (With<Player>, Without<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    creature_query: Query<(Entity, &Transform, &StateMachine<CreatureState>, &Children), (With<Hostile>, Without<Dead>, Without<Stunned>)>,
    fist_query: Query<&Weapon, With<Fist>>,
) {
    let Ok((player_transform, player_hit_collider)) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();
    let player_hit_radius = player_hit_collider.map(|h| h.radius_x.max(h.radius_y)).unwrap_or(0.0);

    for (entity, creature_transform, state_machine, children) in &creature_query {
        // Only detect from Chase state
        if *state_machine.current() != CreatureState::Chase {
            continue;
        }

        let creature_pos = creature_transform.translation.truncate();
        let distance = player_pos.distance(creature_pos);

        // Check if within weapon range
        for child in children.iter() {
            if let Ok(weapon) = fist_query.get(child) {
                if distance < weapon.range() + player_hit_radius {
                    events.write(PlayerInRange {
                        creature: entity,
                        distance,
                    });
                    break;
                }
            }
        }
    }
}

/// Advances attack phases based on WeaponSwing timer progress.
/// - WindUp → Strike when timer reaches hit_delay
/// - Strike → Recovery after hit is applied
/// - Recovery → Chase when swing is complete
pub fn advance_attack_phases(
    mut transitions: MessageWriter<RequestTransition<CreatureState>>,
    creature_query: Query<(Entity, &StateMachine<CreatureState>, &Children)>,
    fist_query: Query<&WeaponSwing, With<Fist>>,
) {
    for (entity, state_machine, children) in &creature_query {
        let CreatureState::Attack(current_phase) = state_machine.current() else {
            continue;
        };

        // Find fist with swing data
        let mut swing_data = None;
        for child in children.iter() {
            if let Ok(swing) = fist_query.get(child) {
                swing_data = Some(swing);
                break;
            }
        }

        let Some(swing) = swing_data else {
            continue;
        };

        match current_phase {
            AttackPhase::WindUp => {
                // Transition to Strike when timer reaches hit_delay
                if swing.timer >= swing.hit_delay {
                    transitions.write(RequestTransition::new(
                        entity,
                        CreatureState::Attack(AttackPhase::Strike),
                    ));
                }
            }
            AttackPhase::Strike => {
                // Transition to Recovery after hit is applied
                if swing.hit_applied {
                    transitions.write(RequestTransition::new(
                        entity,
                        CreatureState::Attack(AttackPhase::Recovery),
                    ));
                }
            }
            AttackPhase::Recovery => {
                // Transition back to Chase when swing is complete
                if swing.timer >= swing.duration {
                    transitions.write(RequestTransition::new(
                        entity,
                        CreatureState::Chase,
                    ));
                }
            }
        }
    }
}
