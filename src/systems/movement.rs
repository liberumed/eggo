use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::{GameAction, Hitstop, InputBindings};

pub fn move_player(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    hitstop: Res<Hitstop>,
    time: Res<Time>,
    mut player_query: Query<(Entity, &mut Transform, &mut PlayerAnimation, &WalkCollider, Option<&mut Sprinting>, Option<&PhaseThrough>), (With<Player>, Without<Dead>, Without<DeathAnimation>, Without<Dashing>)>,
    creatures_query: Query<(&Transform, &WalkCollider, Option<&Dead>), (With<Creature>, Without<Player>)>,
    colliders_query: Query<(&Transform, &StaticCollider), Without<Player>>,
    blocking_query: Query<&Blocking>,
    swing_query: Query<&WeaponSwing, With<PlayerWeapon>>,
    attack_state_query: Query<&PlayerAttackState, With<Player>>,
) {
    // Freeze during hitstop
    if hitstop.is_active() {
        return;
    }

    // Attack commitment: no movement during swing or sprite attack
    let is_swinging = swing_query.iter().next().is_some();
    let is_attacking = attack_state_query.iter().next().is_some();

    let mut input_dir = Vec2::ZERO;

    // Only read input if not attacking (attack commitment)
    if !is_swinging && !is_attacking {
        if bindings.pressed(GameAction::MoveUp, &keyboard, &mouse) {
            input_dir.y += 1.0;
        }
        if bindings.pressed(GameAction::MoveDown, &keyboard, &mouse) {
            input_dir.y -= 1.0;
        }
        if bindings.pressed(GameAction::MoveLeft, &keyboard, &mouse) {
            input_dir.x -= 1.0;
        }
        if bindings.pressed(GameAction::MoveRight, &keyboard, &mouse) {
            input_dir.x += 1.0;
        }
    }

    let dt = time.delta_secs();

    for (entity, mut transform, mut anim, walk_collider, sprinting, phase_through) in &mut player_query {
        let player_radius = Vec2::new(walk_collider.radius_x, walk_collider.radius_y);
        let player_offset_y = walk_collider.offset_y;
        let is_blocking = blocking_query.get(entity).is_ok();
        let is_sprinting = bindings.pressed(GameAction::Sprint, &keyboard, &mouse) && input_dir != Vec2::ZERO;
        let is_phasing = phase_through.is_some();

        // Handle sprint state and ramp-up
        let sprint_multiplier = if is_sprinting {
            let sprint_duration = if let Some(mut sprint) = sprinting {
                sprint.duration += dt;
                sprint.duration
            } else {
                commands.entity(entity).insert(Sprinting { duration: 0.0 });
                0.0
            };
            // Lerp from min to max multiplier over ramp time
            let t = (sprint_duration / SPRINT_RAMP_TIME).min(1.0);
            SPRINT_MIN_MULTIPLIER + t * (SPRINT_MAX_MULTIPLIER - SPRINT_MIN_MULTIPLIER)
        } else {
            // Remove sprinting component when not sprinting
            commands.entity(entity).remove::<Sprinting>();
            1.0
        };

        let speed = if is_blocking {
            PLAYER_SPEED * 0.4
        } else {
            PLAYER_SPEED * sprint_multiplier
        };

        if input_dir != Vec2::ZERO {
            let target_velocity = input_dir.normalize() * speed;
            let diff = target_velocity - anim.velocity;
            let current_speed = anim.velocity.length();

            // Use lower deceleration when slowing from sprint (momentum)
            let is_decelerating = current_speed > speed;
            let accel_amount = if is_decelerating && current_speed > PLAYER_SPEED * 1.1 {
                SPRINT_MOMENTUM_FRICTION
            } else {
                PLAYER_ACCELERATION
            };

            let accel = diff.normalize_or_zero() * accel_amount * dt;
            if accel.length() > diff.length() {
                anim.velocity = target_velocity;
            } else {
                anim.velocity += accel;
            }
        } else if anim.velocity != Vec2::ZERO {
            // Use lower friction when decelerating from sprint (momentum)
            let current_speed = anim.velocity.length();
            let friction_amount = if current_speed > PLAYER_SPEED * 1.1 {
                SPRINT_MOMENTUM_FRICTION
            } else {
                PLAYER_FRICTION
            };
            let friction = anim.velocity.normalize() * friction_amount * dt;
            if friction.length() >= anim.velocity.length() {
                anim.velocity = Vec2::ZERO;
            } else {
                anim.velocity -= friction;
            }
        }

        if anim.velocity == Vec2::ZERO {
            continue;
        }

        let movement = anim.velocity * dt;
        let mut new_pos = Vec2::new(
            transform.translation.x + movement.x,
            transform.translation.y + movement.y,
        );

        let mut blocked_x = false;
        let mut blocked_y = false;

        // Skip creature collision when phasing through (after dash)
        if !is_phasing {
            for (creature_transform, creature_walk, dead) in &creatures_query {
                if dead.is_some() {
                    continue;
                }

                let creature_pos = Vec2::new(
                    creature_transform.translation.x,
                    creature_transform.translation.y + creature_walk.offset_y,
                );
                let creature_radius = Vec2::new(creature_walk.radius_x, creature_walk.radius_y);

                // Ellipse collision - check X movement (at feet level)
                let test_x = Vec2::new(new_pos.x, transform.translation.y + player_offset_y);
                if ellipses_overlap(test_x, player_radius, creature_pos, creature_radius) {
                    blocked_x = true;
                }

                // Ellipse collision - check Y movement (at feet level)
                let test_y = Vec2::new(transform.translation.x, new_pos.y + player_offset_y);
                if ellipses_overlap(test_y, player_radius, creature_pos, creature_radius) {
                    blocked_y = true;
                }
            }
        }

        // Static colliders - push-based ellipse collision (at feet level)
        let mut static_push = Vec2::ZERO;
        for (collider_transform, collider) in &colliders_query {
            let collider_pos = Vec2::new(
                collider_transform.translation.x,
                collider_transform.translation.y + collider.offset_y,
            );
            let collider_radius = Vec2::new(collider.radius_x, collider.radius_y);

            let player_pos = Vec2::new(new_pos.x, new_pos.y + player_offset_y);
            let push = ellipse_push(player_pos, player_radius, collider_pos, collider_radius);
            static_push += push;
        }

        // Apply static collision push
        if static_push != Vec2::ZERO {
            new_pos += static_push;
        }

        if !blocked_x {
            transform.translation.x = new_pos.x;
        } else {
            anim.velocity.x = 0.0;
        }
        if !blocked_y {
            transform.translation.y = new_pos.y;
        } else {
            anim.velocity.y = 0.0;
        }
    }
}

/// Handle dash input and initiation
pub fn handle_dash_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    hitstop: Res<Hitstop>,
    player_query: Query<(Entity, &PlayerAnimation, Option<&DashCooldown>), (With<Player>, Without<Dead>, Without<Dashing>)>,
) {
    if hitstop.is_active() {
        return;
    }

    if !bindings.just_pressed(GameAction::Dash, &keyboard, &mouse) {
        return;
    }

    let Ok((entity, anim, cooldown)) = player_query.single() else { return };

    // Check cooldown
    if let Some(cd) = cooldown {
        if cd.timer > 0.0 {
            return;
        }
    }

    // Determine dash direction (use velocity if moving, else face right)
    let direction = if anim.velocity.length() > 0.1 {
        anim.velocity.normalize()
    } else {
        Vec2::X // Default dash direction
    };

    commands.entity(entity).insert(Dashing {
        direction,
        timer: DASH_DURATION,
    });
    commands.entity(entity).insert(DashCooldown { timer: DASH_COOLDOWN });
}

/// Apply dash movement
pub fn apply_dash(
    mut commands: Commands,
    time: Res<Time>,
    hitstop: Res<Hitstop>,
    mut query: Query<(Entity, &mut Transform, &mut Dashing, &mut PlayerAnimation), With<Player>>,
) {
    if hitstop.is_active() {
        return;
    }

    let dt = time.delta_secs();

    for (entity, mut transform, mut dash, mut anim) in &mut query {
        dash.timer -= dt;

        // Apply dash movement
        let movement = dash.direction * DASH_SPEED * dt;
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;

        // Set velocity to dash direction for smooth transition
        anim.velocity = dash.direction * PLAYER_SPEED;

        if dash.timer <= 0.0 {
            commands.entity(entity).remove::<Dashing>();
            // Add brief phase-through to prevent getting stuck in creatures
            commands.entity(entity).insert(PhaseThrough { timer: 0.15 });
        }
    }
}

/// Tick phase-through timer
pub fn tick_phase_through(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PhaseThrough)>,
) {
    let dt = time.delta_secs();
    for (entity, mut phase) in &mut query {
        phase.timer -= dt;
        if phase.timer <= 0.0 {
            commands.entity(entity).remove::<PhaseThrough>();
        }
    }
}

/// Tick dash cooldown
pub fn tick_dash_cooldown(
    time: Res<Time>,
    mut query: Query<&mut DashCooldown>,
) {
    let dt = time.delta_secs();
    for mut cooldown in &mut query {
        cooldown.timer = (cooldown.timer - dt).max(0.0);
    }
}

pub fn apply_knockback(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Knockback, Option<&Health>, Option<&DeathAnimation>), Or<(With<Player>, With<Creature>)>>,
) {
    for (entity, mut transform, mut knockback, health, death_anim) in &mut query {
        knockback.timer += time.delta_secs();

        let decay = (1.0 - knockback.timer * 3.0).max(0.0);
        let movement = knockback.velocity * decay * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;

        if knockback.timer > 0.3 {
            commands.entity(entity).remove::<Knockback>();
            // Only start death animation if not already dying and health is depleted
            if death_anim.is_none() {
                if let Some(h) = health {
                    if h.0 <= 0 {
                        commands.entity(entity).insert(DeathAnimation { timer: 0.0, stage: 0 });
                    }
                }
            }
        }
    }
}

/// Push creatures away from player and each other
pub fn apply_collision_push(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Dead>, Without<Creature>)>,
    mut creatures_query: Query<(Entity, &mut Transform), (With<Creature>, Without<Dead>, Without<Player>)>,
) {
    let dt = time.delta_secs();

    // Collect creature positions for creature-creature push
    let creature_positions: Vec<(Entity, Vec2)> = creatures_query
        .iter()
        .map(|(e, t)| (e, Vec2::new(t.translation.x, t.translation.y)))
        .collect();

    // Get player position
    let player_pos = player_query
        .single()
        .map(|t| Vec2::new(t.translation.x, t.translation.y))
        .ok();

    for (entity, mut transform) in &mut creatures_query {
        let creature_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let mut push = Vec2::ZERO;

        // 1. Player pushes creatures
        if let Some(player_pos) = player_pos {
            let diff = creature_pos - player_pos;
            let dist = diff.length();

            if dist < PUSH_RADIUS && dist > 0.1 {
                let push_dir = diff.normalize();
                let overlap = (PUSH_RADIUS - dist) / PUSH_RADIUS;
                push += push_dir * overlap * PUSH_STRENGTH * dt;
            }
        }

        // 2. Creatures push each other
        for (other_entity, other_pos) in &creature_positions {
            if *other_entity == entity {
                continue;
            }

            let diff = creature_pos - *other_pos;
            let dist = diff.length();

            if dist < PUSH_RADIUS && dist > 0.1 {
                let push_dir = diff.normalize();
                let overlap = (PUSH_RADIUS - dist) / PUSH_RADIUS;
                push += push_dir * overlap * PUSH_STRENGTH * 0.5 * dt;
            }
        }

        // Apply push
        transform.translation.x += push.x;
        transform.translation.y += push.y;
    }
}
