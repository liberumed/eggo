use bevy::prelude::*;

use crate::combat::snap_to_cardinal;
use crate::inventory::weapons::{Fist, Weapon, WeaponSwing};
use crate::core::{Dead, DeathAnimation, GameConfig, HitCollider, Stunned};
use crate::player::Player;
use crate::state_machine::{AttackPhase, RequestTransition, StateEntered, StateExited, StateMachine};
use crate::constants::Z_UI_WORLD;
use super::{AlertIndicator, AttackOffset, CardinalAttacks, Creature, CreatureState, Hostile, PlayerInRange};

pub fn on_attack_windup_enter(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut events: MessageReader<StateEntered<CreatureState>>,
    player_query: Query<&Transform, With<Player>>,
    creature_query: Query<(&Transform, &Children, Option<&CardinalAttacks>)>,
    fist_query: Query<(Entity, &Weapon), With<Fist>>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();

    for event in events.read() {
        if !matches!(event.state, CreatureState::Attack(AttackPhase::WindUp)) {
            continue;
        }

        let Ok((creature_transform, children, cardinal_attacks)) = creature_query.get(event.entity) else {
            continue;
        };

        let creature_pos = creature_transform.translation.truncate();
        let dir = player_pos - creature_pos;
        let raw_angle = dir.y.atan2(dir.x);

        let base_angle = Some(if cardinal_attacks.is_some() {
            snap_to_cardinal(raw_angle)
        } else {
            raw_angle
        });

        for child in children.iter() {
            if let Ok((fist_entity, weapon)) = fist_query.get(child) {
                let duration = weapon.swing_duration();
                commands.entity(fist_entity).insert(WeaponSwing {
                    timer: 0.0,
                    duration,
                    base_angle,
                    attack_type: weapon.attack_type,
                    hit_delay: duration * config.attack_hit_delay_percent,
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

pub fn on_creature_stunned(
    mut transitions: MessageWriter<RequestTransition<CreatureState>>,
    query: Query<(Entity, &StateMachine<CreatureState>), Added<Stunned>>,
) {
    for (entity, state_machine) in &query {
        if *state_machine.current() != CreatureState::Stunned {
            transitions.write(RequestTransition::new(entity, CreatureState::Stunned));
        }
    }
}

pub fn on_stun_recovered(
    mut transitions: MessageWriter<RequestTransition<CreatureState>>,
    mut removed: RemovedComponents<Stunned>,
    query: Query<(Entity, &StateMachine<CreatureState>, Option<&super::Activated>, Option<&super::PatrolOrigin>), With<Hostile>>,
) {
    for entity in removed.read() {
        if let Ok((_, state_machine, activated, patrol_origin)) = query.get(entity) {
            if *state_machine.current() != CreatureState::Stunned {
                continue;
            }
            let target = if activated.is_some() {
                CreatureState::Chase
            } else if patrol_origin.is_some() {
                CreatureState::Patrol
            } else {
                CreatureState::Chase
            };
            transitions.write(RequestTransition::new(entity, target));
        }
    }
}

pub fn on_hostile_start_patrol(
    mut transitions: MessageWriter<RequestTransition<CreatureState>>,
    query: Query<(Entity, &StateMachine<CreatureState>), (With<Hostile>, With<super::PatrolOrigin>, Without<super::Activated>, Added<StateMachine<CreatureState>>)>,
) {
    for (entity, state_machine) in &query {
        if *state_machine.current() == CreatureState::Idle {
            transitions.write(RequestTransition::new(entity, CreatureState::Patrol));
        }
    }
}

pub fn on_deactivated_to_patrol(
    mut transitions: MessageWriter<RequestTransition<CreatureState>>,
    mut removed: RemovedComponents<super::Activated>,
    query: Query<(Entity, &StateMachine<CreatureState>), (With<Hostile>, With<super::PatrolOrigin>)>,
) {
    for entity in removed.read() {
        if let Ok((_, state_machine)) = query.get(entity) {
            if *state_machine.current() == CreatureState::Chase {
                transitions.write(RequestTransition::new(entity, CreatureState::Patrol));
            }
        }
    }
}

pub fn on_activated_to_chase(
    mut transitions: MessageWriter<RequestTransition<CreatureState>>,
    query: Query<(Entity, &StateMachine<CreatureState>), (With<Hostile>, Added<super::Activated>)>,
) {
    for (entity, state_machine) in &query {
        if *state_machine.current() == CreatureState::Patrol {
            transitions.write(RequestTransition::new(entity, CreatureState::Chase));
        }
    }
}

pub fn on_alert_enter(
    mut commands: Commands,
    mut events: MessageReader<StateEntered<CreatureState>>,
    creature_query: Query<&Transform, With<Creature>>,
) {
    for event in events.read() {
        if event.state != CreatureState::Alert {
            continue;
        }

        let Ok(transform) = creature_query.get(event.entity) else { continue };

        let indicator = commands.spawn((
            Text2d::new("!"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgb(1.0, 0.8, 0.0)),
            Transform::from_xyz(transform.translation.x, transform.translation.y + 35.0, Z_UI_WORLD + 1.0),
        )).id();

        commands.entity(event.entity).insert(AlertIndicator(indicator));
    }
}

pub fn on_alert_exit(
    mut commands: Commands,
    mut events: MessageReader<StateExited<CreatureState>>,
    creature_query: Query<&AlertIndicator>,
) {
    for event in events.read() {
        if event.state != CreatureState::Alert {
            continue;
        }

        let Ok(indicator) = creature_query.get(event.entity) else { continue };
        commands.entity(indicator.0).despawn();
        commands.entity(event.entity).remove::<AlertIndicator>();
    }
}

/// Detects when creatures in Chase state are within weapon range of the player.
/// Emits PlayerInRange event for other systems to react to.
pub fn detect_player_proximity(
    mut events: MessageWriter<PlayerInRange>,
    player_query: Query<(&Transform, Option<&HitCollider>), (With<Player>, Without<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    creature_query: Query<(Entity, &Transform, &StateMachine<CreatureState>, &Children, Option<&AttackOffset>), (With<Hostile>, Without<Dead>, Without<Stunned>)>,
    fist_query: Query<&Weapon, With<Fist>>,
) {
    let Ok((player_transform, player_hit_collider)) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();

    for (entity, creature_transform, state_machine, children, attack_offset) in &creature_query {
        if *state_machine.current() != CreatureState::Chase {
            continue;
        }

        let creature_pos = creature_transform.translation.truncate();

        let offset_y = attack_offset.map(|o| o.0).unwrap_or(0.0);
        let attack_origin = Vec2::new(creature_pos.x, creature_pos.y + offset_y);

        let min_distance = player_hit_collider
            .map(|collider| {
                collider.circles.iter()
                    .map(|c| (player_pos + c.offset).distance(attack_origin) - c.radius)
                    .fold(f32::MAX, f32::min)
            })
            .unwrap_or_else(|| player_pos.distance(attack_origin));

        for child in children.iter() {
            if let Ok(weapon) = fist_query.get(child) {
                if min_distance < weapon.range() {
                    events.write(PlayerInRange {
                        creature: entity,
                        distance: min_distance,
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
                // Transition to Cooldown when swing is complete
                if swing.timer >= swing.duration {
                    transitions.write(RequestTransition::new(
                        entity,
                        CreatureState::Cooldown,
                    ));
                }
            }
        }
    }
}

/// Transitions creatures from Cooldown back to Chase after the cooldown expires
pub fn advance_cooldown_state(
    config: Res<GameConfig>,
    mut transitions: MessageWriter<RequestTransition<CreatureState>>,
    query: Query<(Entity, &StateMachine<CreatureState>), With<Hostile>>,
) {
    for (entity, state_machine) in &query {
        if *state_machine.current() == CreatureState::Cooldown {
            if state_machine.time_in_state() >= config.attack_cooldown_duration {
                transitions.write(RequestTransition::new(entity, CreatureState::Chase));
            }
        }
    }
}
