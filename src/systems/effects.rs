use bevy::prelude::*;

use crate::components::*;

pub fn animate_blood(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut BloodParticle)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle) in &mut query {
        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        particle.velocity.y -= 150.0 * dt;
        particle.velocity *= 0.98;

        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        let fade = (particle.lifetime / 0.5).min(1.0);
        transform.scale = Vec3::splat(fade * 0.8 + 0.2);

        transform.rotation *= Quat::from_rotation_z(dt * 3.0);
    }
}

pub fn animate_resource_balls(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ResourceBall), Without<MagnetizedBall>>,
) {
    let dt = time.delta_secs();
    let egg_a = 6.5;
    let egg_b = 10.0;

    for (mut transform, mut ball) in &mut query {
        let mut pos = Vec2::new(transform.translation.x, transform.translation.y);
        pos += ball.velocity * dt;

        let norm_x = pos.x / egg_a;
        let norm_y = pos.y / egg_b;
        let dist_sq = norm_x * norm_x + norm_y * norm_y;

        if dist_sq > 1.0 {
            let normal = Vec2::new(pos.x / (egg_a * egg_a), pos.y / (egg_b * egg_b)).normalize();
            ball.velocity = ball.velocity - 2.0 * ball.velocity.dot(normal) * normal;
            let scale = 1.0 / dist_sq.sqrt();
            pos.x *= scale;
            pos.y *= scale;
        }

        ball.velocity.y -= 5.0 * dt;

        let wobble = (time.elapsed_secs() * 3.0 + pos.x * 0.5).sin() * 2.0;
        ball.velocity.x += wobble * dt;

        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

pub fn animate_magnetized_balls(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<MagnetizedBall>)>,
    mut query: Query<(Entity, &mut Transform, &mut DespawnTimer), (With<MagnetizedBall>, With<ResourceBall>)>,
) {
    let dt = time.delta_secs();
    let player_pos = player_query.single().map(|t| Vec2::new(t.translation.x, t.translation.y)).unwrap_or(Vec2::ZERO);

    for (entity, mut transform, mut timer) in &mut query {
        timer.0 -= dt;

        if timer.0 <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let ball_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let dir = (player_pos - ball_pos).normalize_or_zero();
        let elapsed = 3.0 - timer.0;
        let speed = 80.0 + elapsed * 40.0;
        let new_pos = ball_pos + dir * speed * dt;

        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;

        let fade = (timer.0 / 3.0).powf(0.5);
        transform.scale = Vec3::splat(fade.max(0.1));
    }
}
