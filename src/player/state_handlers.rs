use bevy::prelude::*;

use crate::inventory::AttackType;
use crate::inventory::weapons::{Drawn, PlayerWeapon, Weapon, WeaponSwing};
use crate::constants::ATTACK_HIT_DELAY_PERCENT;
use crate::core::{Dead, DeathAnimation, GameAction, InputBindings};
use crate::state_machine::{AttackPhase, RequestTransition, StateEntered, StateExited, StateMachine};
use super::{
    DashCooldown, DashInputDetected, AttackInputDetected, MovementInputDetected,
    PhaseThrough, Player, PlayerAnimation, PlayerState,
};

pub fn detect_movement_input(
    mut events: MessageWriter<MovementInputDetected>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    query: Query<(Entity, &StateMachine<PlayerState>), (With<Player>, Without<Dead>, Without<DeathAnimation>)>,
) {
    let Ok((entity, state)) = query.single() else { return };

    match state.current() {
        PlayerState::Idle | PlayerState::Moving => {}
        _ => return,
    }

    let mut direction = Vec2::ZERO;
    if bindings.pressed(GameAction::MoveUp, &keyboard, &mouse) {
        direction.y += 1.0;
    }
    if bindings.pressed(GameAction::MoveDown, &keyboard, &mouse) {
        direction.y -= 1.0;
    }
    if bindings.pressed(GameAction::MoveLeft, &keyboard, &mouse) {
        direction.x -= 1.0;
    }
    if bindings.pressed(GameAction::MoveRight, &keyboard, &mouse) {
        direction.x += 1.0;
    }

    events.write(MovementInputDetected {
        player: entity,
        direction,
    });
}

pub fn detect_dash_input(
    mut events: MessageWriter<DashInputDetected>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    query: Query<(Entity, &PlayerAnimation, &StateMachine<PlayerState>, Option<&DashCooldown>), (With<Player>, Without<Dead>, Without<DeathAnimation>)>,
) {
    if !bindings.just_pressed(GameAction::Dash, &keyboard, &mouse) {
        return;
    }

    let Ok((entity, anim, state, cooldown)) = query.single() else { return };

    match state.current() {
        PlayerState::Idle | PlayerState::Moving => {}
        _ => return,
    }

    if let Some(cd) = cooldown {
        if cd.timer > 0.0 {
            return;
        }
    }

    let direction = if anim.velocity.length() > 0.1 {
        anim.velocity.normalize()
    } else {
        Vec2::X
    };

    events.write(DashInputDetected {
        player: entity,
        direction,
    });
}

pub fn detect_attack_input(
    mut commands: Commands,
    mut events: MessageWriter<AttackInputDetected>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    query: Query<(Entity, &Transform, &StateMachine<PlayerState>), (With<Player>, Without<Dead>, Without<DeathAnimation>)>,
    weapon_query: Query<(Entity, &Weapon, Option<&Drawn>), With<PlayerWeapon>>,
) {
    if !bindings.just_pressed(GameAction::Attack, &keyboard, &mouse) {
        return;
    }

    let Ok((entity, player_transform, state)) = query.single() else { return };

    match state.current() {
        PlayerState::Idle | PlayerState::Moving => {}
        _ => return,
    }

    let Ok((weapon_entity, weapon, drawn)) = weapon_query.single() else { return };

    // First click draws weapon (auto-draw behavior)
    if drawn.is_none() {
        commands.entity(weapon_entity).insert(Drawn);
        // Smash weapons don't show mesh (use sprite animation)
        if weapon.attack_type != AttackType::Smash {
            commands.entity(weapon_entity).insert(Visibility::Inherited);
        }
        return;
    }

    let facing_right = if let Some(world_pos) = get_cursor_world_pos(&windows, &camera_query) {
        world_pos.x >= player_transform.translation.x
    } else {
        true
    };

    events.write(AttackInputDetected {
        player: entity,
        facing_right,
    });
}

fn get_cursor_world_pos(
    windows: &Query<&Window>,
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) -> Option<Vec2> {
    let window = windows.iter().next()?;
    let cursor_pos = window.cursor_position()?;
    let (camera, camera_transform) = camera_query.iter().next()?;
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

pub fn handle_movement_input(
    mut transitions: MessageWriter<RequestTransition<PlayerState>>,
    mut events: MessageReader<MovementInputDetected>,
    query: Query<&StateMachine<PlayerState>, With<Player>>,
) {
    for event in events.read() {
        let Ok(state) = query.get(event.player) else { continue };

        let has_input = event.direction != Vec2::ZERO;

        match state.current() {
            PlayerState::Idle if has_input => {
                transitions.write(RequestTransition::new(event.player, PlayerState::Moving));
            }
            PlayerState::Moving if !has_input => {
            }
            _ => {}
        }
    }
}

pub fn handle_dash_input(
    mut commands: Commands,
    mut transitions: MessageWriter<RequestTransition<PlayerState>>,
    mut events: MessageReader<DashInputDetected>,
) {
    use crate::constants::{DASH_COOLDOWN, DASH_DURATION};

    for event in events.read() {
        commands.entity(event.player).insert(PlayerDashing {
            direction: event.direction,
            timer: DASH_DURATION,
        });
        commands.entity(event.player).insert(DashCooldown { timer: DASH_COOLDOWN });
        transitions.write(RequestTransition::new(event.player, PlayerState::Dashing));
    }
}

pub fn handle_attack_input(
    mut transitions: MessageWriter<RequestTransition<PlayerState>>,
    mut events: MessageReader<AttackInputDetected>,
    mut commands: Commands,
) {
    for event in events.read() {
        commands.entity(event.player).insert(PlayerAttacking {
            facing_right: event.facing_right,
        });
        transitions.write(RequestTransition::new(
            event.player,
            PlayerState::Attacking(AttackPhase::WindUp),
        ));
    }
}

#[derive(Component)]
pub struct PlayerDashing {
    pub direction: Vec2,
    pub timer: f32,
}

#[derive(Component)]
pub struct PlayerAttacking {
    pub facing_right: bool,
}

pub fn on_dashing_exit(
    mut commands: Commands,
    mut events: MessageReader<StateExited<PlayerState>>,
) {
    for event in events.read() {
        if event.state != PlayerState::Dashing {
            continue;
        }

        commands.entity(event.entity).remove::<PlayerDashing>();
        commands.entity(event.entity).insert(PhaseThrough { timer: 0.15 });
    }
}

pub fn on_attacking_windup_enter(
    mut commands: Commands,
    mut events: MessageReader<StateEntered<PlayerState>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    query: Query<(&Transform, &Children, Option<&PlayerAttacking>), With<Player>>,
    weapon_query: Query<(Entity, &Weapon, &Transform), With<PlayerWeapon>>,
) {
    for event in events.read() {
        if !matches!(event.state, PlayerState::Attacking(AttackPhase::WindUp)) {
            continue;
        }

        let Ok((player_transform, children, attacking)) = query.get(event.entity) else { continue };

        // PlayerAttacking may not be inserted yet due to command deferral
        // Fall back to calculating from cursor position
        let facing_right = attacking.map(|a| a.facing_right).unwrap_or_else(|| {
            get_cursor_world_pos(&windows, &camera_query)
                .map(|pos| pos.x >= player_transform.translation.x)
                .unwrap_or(true)
        });

        for child in children.iter() {
            if let Ok((weapon_entity, weapon, weapon_transform)) = weapon_query.get(child) {
                let duration = weapon.swing_duration();

                if weapon.attack_type == AttackType::Smash {
                    commands.entity(event.entity).insert(PlayerSmashAttack {
                        timer: 0.0,
                        duration,
                        hit_applied: false,
                        facing_right,
                    });
                } else {
                    let (_, angle) = weapon_transform.rotation.to_axis_angle();
                    let base_angle = if weapon_transform.rotation.z < 0.0 { -angle } else { angle };

                    commands.entity(weapon_entity).insert(WeaponSwing {
                        timer: 0.0,
                        duration,
                        base_angle: Some(base_angle),
                        attack_type: weapon.attack_type,
                        hit_delay: duration * ATTACK_HIT_DELAY_PERCENT,
                        hit_applied: false,
                    });
                }
                break;
            }
        }
    }
}

#[derive(Component)]
pub struct PlayerSmashAttack {
    pub timer: f32,
    pub duration: f32,
    pub hit_applied: bool,
    pub facing_right: bool,
}

pub fn on_attacking_exit(
    mut commands: Commands,
    mut events: MessageReader<StateExited<PlayerState>>,
    query: Query<(&StateMachine<PlayerState>, &Children), With<Player>>,
    weapon_query: Query<Entity, With<PlayerWeapon>>,
) {
    for event in events.read() {
        if !matches!(event.state, PlayerState::Attacking(_)) {
            continue;
        }

        let Ok((state, children)) = query.get(event.entity) else { continue };

        if matches!(state.current(), PlayerState::Attacking(_)) {
            continue;
        }

        commands.entity(event.entity).remove::<PlayerAttacking>();
        commands.entity(event.entity).remove::<PlayerSmashAttack>();

        for child in children.iter() {
            if weapon_query.get(child).is_ok() {
                commands.entity(child).remove::<WeaponSwing>();
            }
        }
    }
}

pub fn on_idle_enter(
    mut events: MessageReader<StateEntered<PlayerState>>,
    mut query: Query<&mut PlayerAnimation, With<Player>>,
) {
    for event in events.read() {
        if event.state != PlayerState::Idle {
            continue;
        }

        if let Ok(mut anim) = query.get_mut(event.entity) {
            anim.velocity = Vec2::ZERO;
        }
    }
}
