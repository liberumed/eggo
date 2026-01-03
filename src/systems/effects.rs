use bevy::prelude::*;

use crate::components::*;
use crate::resources::{Hitstop, ScreenShake};

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

/// Tick hitstop timer
pub fn tick_hitstop(
    time: Res<Time>,
    mut hitstop: ResMut<Hitstop>,
) {
    hitstop.tick(time.delta_secs());
}

/// Tick screen shake timer
pub fn tick_screen_shake(
    time: Res<Time>,
    mut screen_shake: ResMut<ScreenShake>,
) {
    screen_shake.tick(time.delta_secs());
}

/// Animate dust particles (fade out and rise)
pub fn animate_dust(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut DustParticle)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut dust) in &mut query {
        dust.lifetime -= dt;

        if dust.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Slowly rise and expand
        transform.translation.y += 8.0 * dt;
        let fade = dust.lifetime / 0.4;
        transform.scale = Vec3::splat(1.0 + (1.0 - fade) * 0.5) * fade;
    }
}

/// Animate hit highlight (red flash on creatures with MeshMaterial2d)
pub fn animate_hit_highlight(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(Entity, &mut HitHighlight, &mut MeshMaterial2d<ColorMaterial>)>,
) {
    for (entity, mut highlight, mut material_handle) in &mut query {
        let dt = time.delta_secs();

        // On first frame, store original material and swap to red
        if highlight.original_material.is_none() {
            highlight.original_material = Some(material_handle.0.clone());
            // Create a bright red flash material
            let flash_material = materials.add(Color::srgb(1.0, 0.3, 0.3));
            material_handle.0 = flash_material;
        }

        highlight.timer += dt;

        if highlight.timer >= highlight.duration {
            // Restore original material and remove component
            if let Some(original) = highlight.original_material.take() {
                material_handle.0 = original;
            }
            commands.entity(entity).remove::<HitHighlight>();
        } else {
            // Pulse the red intensity during flash
            let progress = highlight.timer / highlight.duration;
            let intensity = 1.0 - progress; // Fade from bright to normal
            let flash_color = Color::srgb(1.0, 0.3 + intensity * 0.3, 0.3 + intensity * 0.3);
            material_handle.0 = materials.add(flash_color);
        }
    }
}

/// Animate hit highlight for sprites (red flash on player)
pub fn animate_sprite_hit_highlight(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitHighlight, &mut Sprite), Without<MeshMaterial2d<ColorMaterial>>>,
) {
    for (entity, mut highlight, mut sprite) in &mut query {
        let dt = time.delta_secs();

        highlight.timer += dt;

        if highlight.timer >= highlight.duration {
            // Restore to white and remove component
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<HitHighlight>();
        } else {
            // Flash red, fading back to normal
            let progress = highlight.timer / highlight.duration;
            let intensity = 1.0 - progress;
            sprite.color = Color::srgb(1.0, 0.3 + intensity * 0.4, 0.3 + intensity * 0.4);
        }
    }
}

/// Spawn dust particles behind sprinting player
pub fn spawn_sprint_dust(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(&Transform, &Sprinting, &PlayerAnimation), With<Player>>,
) {
    for (transform, sprinting, anim) in &query {
        // Only spawn dust when at higher speeds (ramped up)
        if sprinting.duration < 0.2 || anim.velocity.length() < 50.0 {
            continue;
        }

        // Spawn rate increases with sprint duration
        let spawn_chance = (sprinting.duration * 2.0).min(1.0) * 0.3;
        if rand::random::<f32>() > spawn_chance {
            continue;
        }

        let offset = Vec2::new(
            rand::random::<f32>() * 6.0 - 3.0,
            rand::random::<f32>() * 2.0 - 4.0,
        );

        commands.spawn((
            DustParticle { lifetime: 0.4 },
            Mesh2d(meshes.add(Circle::new(2.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(0.6, 0.55, 0.45, 0.6)))),
            Transform::from_xyz(
                transform.translation.x + offset.x,
                transform.translation.y + offset.y,
                crate::constants::Z_PARTICLE,
            ),
        ));
    }
}
