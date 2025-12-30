use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;

pub fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut PlayerAnimation), (With<Player>, Without<Dead>)>,
    creatures_query: Query<(&Transform, Option<&Dead>), (With<Creature>, Without<Player>)>,
) {
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let collision_distance = COLLISION_RADIUS * 1.5;

    for (mut transform, mut anim) in &mut player_query {
        if direction == Vec2::ZERO {
            anim.velocity = Vec2::ZERO;
            continue;
        }

        let dir_normalized = direction.normalize();
        let velocity = dir_normalized * PLAYER_SPEED * time.delta_secs();
        let new_pos = Vec2::new(
            transform.translation.x + velocity.x,
            transform.translation.y + velocity.y,
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

        let mut actual_velocity = Vec2::ZERO;
        if !blocked_x {
            transform.translation.x = new_pos.x;
            actual_velocity.x = velocity.x;
        }
        if !blocked_y {
            transform.translation.y = new_pos.y;
            actual_velocity.y = velocity.y;
        }
        anim.velocity = actual_velocity;
    }
}

pub fn apply_knockback(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Knockback), With<Player>>,
) {
    for (entity, mut transform, mut knockback) in &mut query {
        knockback.timer += time.delta_secs();

        let decay = (1.0 - knockback.timer * 3.0).max(0.0);
        let movement = knockback.velocity * decay * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;

        if knockback.timer > 0.3 {
            commands.entity(entity).remove::<Knockback>();
        }
    }
}
