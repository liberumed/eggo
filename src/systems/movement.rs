use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;

pub fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(Entity, &mut Transform, &mut PlayerAnimation), (With<Player>, Without<Dead>, Without<DeathAnimation>)>,
    creatures_query: Query<(&Transform, Option<&Dead>), (With<Creature>, Without<Player>)>,
    blocking_query: Query<&Blocking>,
) {
    let mut input_dir = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        input_dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        input_dir.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        input_dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        input_dir.x += 1.0;
    }

    let collision_distance = COLLISION_RADIUS * 1.5;
    let dt = time.delta_secs();

    for (entity, mut transform, mut anim) in &mut player_query {
        let is_blocking = blocking_query.get(entity).is_ok();
        let speed = if is_blocking { PLAYER_SPEED * 0.4 } else { PLAYER_SPEED };

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

pub fn apply_knockback(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Knockback, &Health), (With<Player>, Without<DeathAnimation>)>,
) {
    for (entity, mut transform, mut knockback, health) in &mut query {
        knockback.timer += time.delta_secs();

        let decay = (1.0 - knockback.timer * 3.0).max(0.0);
        let movement = knockback.velocity * decay * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;

        if knockback.timer > 0.3 {
            commands.entity(entity).remove::<Knockback>();
            if health.0 <= 0 {
                commands.entity(entity).insert(DeathAnimation { timer: 0.0, stage: 0 });
            }
        }
    }
}
