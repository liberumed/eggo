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
    mut player_query: Query<(Entity, &mut Transform, &mut PlayerAnimation, Option<&mut Sprinting>), (With<Player>, Without<Dead>, Without<DeathAnimation>, Without<Dashing>)>,
    creatures_query: Query<(&Transform, Option<&Dead>), (With<Creature>, Without<Player>)>,
    blocking_query: Query<&Blocking>,
    swing_query: Query<&WeaponSwing, With<PlayerWeapon>>,
) {
    // Freeze during hitstop
    if hitstop.is_active() {
        return;
    }

    // Attack commitment: no movement during swing
    let is_swinging = swing_query.iter().next().is_some();

    let mut input_dir = Vec2::ZERO;

    // Only read input if not swinging (attack commitment)
    if !is_swinging {
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

    let collision_distance = COLLISION_RADIUS * 1.5;
    let dt = time.delta_secs();

    for (entity, mut transform, mut anim, sprinting) in &mut player_query {
        let is_blocking = blocking_query.get(entity).is_ok();
        let is_sprinting = bindings.pressed(GameAction::Sprint, &keyboard, &mouse) && input_dir != Vec2::ZERO;

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
            let accel = diff.normalize_or_zero() * PLAYER_ACCELERATION * dt;
            if accel.length() > diff.length() {
                anim.velocity = target_velocity;
            } else {
                anim.velocity += accel;
            }
        } else if anim.velocity != Vec2::ZERO {
            let friction = anim.velocity.normalize() * PLAYER_FRICTION * dt;
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
        let new_pos = Vec2::new(
            transform.translation.x + movement.x,
            transform.translation.y + movement.y,
        );

        let mut blocked_x = false;
        let mut blocked_y = false;

        for (creature_transform, dead) in &creatures_query {
            if dead.is_some() {
                continue;
            }

            let creature_pos = Vec2::new(creature_transform.translation.x, creature_transform.translation.y);

            let test_x = Vec2::new(new_pos.x, transform.translation.y);
            if test_x.distance(creature_pos) < collision_distance {
                blocked_x = true;
            }

            let test_y = Vec2::new(transform.translation.x, new_pos.y);
            if test_y.distance(creature_pos) < collision_distance {
                blocked_y = true;
            }
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
