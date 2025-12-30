use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::Stats;
use crate::spawners::CharacterAssets;

pub fn animate_player(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PlayerAnimation), With<Player>>,
) {
    for (mut transform, mut anim) in &mut query {
        anim.time += time.delta_secs();
        let t = anim.time;

        let is_moving = anim.velocity.length() > 0.1;

        if is_moving {
            let speed_factor = (anim.velocity.length() / (PLAYER_SPEED * time.delta_secs())).min(1.0);
            let bounce = (t * 20.0).sin() * 0.12 * speed_factor;
            let funk = (t * 7.0).cos() * 0.04 * speed_factor;
            let squash_x = 1.0 + bounce + funk;
            let squash_y = 1.0 - bounce * 0.7 - funk * 0.5;

            transform.scale = Vec3::new(squash_x, squash_y, 1.0);

            let tilt = anim.velocity.x * 0.002;
            let dance_tilt = (t * 15.0).sin() * 0.03 * speed_factor;
            transform.rotation = Quat::from_rotation_z(-tilt + dance_tilt);
        } else {
            let swing = (t * 3.5).sin();
            let swing_snap = swing.signum() * swing.abs().powf(0.6) * 0.12;

            let groove = ((t * 4.0).sin() * 0.5 + 0.5).powf(1.5) * 0.08;
            let funk = (t * 1.2).cos() * 0.05;
            let jazz = ((t * 5.0).sin() * (t * 1.5).cos()) * 0.03;

            transform.scale.x = 1.0 + groove - funk * 0.5 + jazz;
            transform.scale.y = 1.0 - groove * 0.7 + funk * 0.3 - jazz * 0.5;
            transform.rotation = Quat::from_rotation_z(swing_snap + jazz * 1.5);
        }
    }
}

pub fn animate_creatures(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &CreatureAnimation), (With<Creature>, Without<Dead>)>,
) {
    let t = time.elapsed_secs();

    for (mut transform, anim) in &mut query {
        let phase = anim.phase;
        let speed = anim.speed;
        let amp = anim.amplitude * 2.5;

        let swing = (t * speed + phase).sin();
        let swing_snap = swing.signum() * swing.abs().powf(0.7) * amp * 1.5;

        let groove = ((t * speed * 2.0 + phase).sin() * 0.5 + 0.5).powf(1.5);
        let bounce = groove * amp * 0.8;

        let funk = (t * speed * 0.5 + phase * 1.3).cos() * amp * 0.6;
        let jazz = ((t * speed * 3.0 + phase).sin() * (t * speed * 0.7).cos()) * amp * 0.3;

        transform.scale.x = 1.0 + bounce - funk * 0.5 + jazz;
        transform.scale.y = 1.0 - bounce * 0.7 + funk * 0.3 - jazz * 0.5;
        transform.rotation = Quat::from_rotation_z(swing_snap + jazz * 2.0);
    }
}

pub fn animate_weapon_swing(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut WeaponSwing)>,
) {
    for (entity, mut transform, mut swing) in &mut query {
        swing.timer += time.delta_secs();
        let t = swing.timer;

        if t < swing.duration {
            let progress = t / swing.duration;

            if let Some(base_angle) = swing.base_angle {
                let (thrust_scale, rotation_offset) = if progress < 0.15 {
                    let p = progress / 0.15;
                    (1.0 - p * 0.2, p * 0.15)
                } else if progress < 0.4 {
                    let p = (progress - 0.15) / 0.25;
                    (0.8 + p * 0.8, 0.15 - p * 0.15)
                } else if progress < 0.6 {
                    let p = (progress - 0.4) / 0.2;
                    (1.6 - p * 0.1, -p * 0.1)
                } else {
                    let p = (progress - 0.6) / 0.4;
                    (1.5 - p * 0.5, -0.1 + p * 0.1)
                };
                transform.scale = Vec3::new(thrust_scale, 1.0, 1.0);
                transform.rotation = Quat::from_rotation_z(base_angle + rotation_offset);
            } else {
                let punch_scale = if progress < 0.3 {
                    1.0 + (progress / 0.3) * 0.5
                } else {
                    1.5 - 0.5 * ((progress - 0.3) / 0.7)
                };
                transform.scale = Vec3::splat(punch_scale);
            }
        } else {
            transform.scale = Vec3::ONE;
            if let Some(base_angle) = swing.base_angle {
                transform.rotation = Quat::from_rotation_z(base_angle);
            }
            commands.entity(entity).remove::<WeaponSwing>();
        }
    }
}

pub fn animate_death(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<CharacterAssets>,
    mut stats: ResMut<Stats>,
    mut query: Query<(Entity, &Transform, &mut DeathAnimation, &Loot, &Children), With<Creature>>,
    ball_query: Query<&Transform, With<ResourceBall>>,
) {
    let mut balls_to_magnetize: Vec<(Entity, Vec3)> = Vec::new();

    for (entity, transform, mut death, loot, children) in &mut query {
        death.timer += time.delta_secs();
        let t = death.timer;

        match death.stage {
            0 => {
                let shake = (t * 60.0).sin() * 0.15 * (1.0 - t * 2.0).max(0.0);
                let expand = 1.0 + t * 0.5;

                commands.entity(entity).insert(Transform {
                    translation: Vec3::new(transform.translation.x, transform.translation.y, Z_DEAD),
                    rotation: Quat::from_rotation_z(shake),
                    scale: Vec3::new(expand, expand * 0.9, 1.0),
                });

                if t > DEATH_EXPAND_DURATION {
                    death.stage = 1;
                    death.timer = 0.0;

                    let parent_pos = transform.translation;
                    let child_list: Vec<Entity> = children.iter().collect();
                    for child in child_list {
                        if let Ok(ball_transform) = ball_query.get(child) {
                            let world_pos = Vec3::new(
                                parent_pos.x + ball_transform.translation.x,
                                parent_pos.y + ball_transform.translation.y,
                                Z_PARTICLE,
                            );
                            balls_to_magnetize.push((child, world_pos));
                        }
                    }

                    commands.entity(entity).insert((
                        Dead,
                        DespawnTimer(CORPSE_LIFETIME),
                        MeshMaterial2d(assets.dead_material.clone()),
                    ));

                    if loot.philosophy { stats.philosophy += 1; }
                    if loot.nature_study { stats.nature_study += 1; }
                    if loot.wisdom { stats.wisdom += 1; }
                }
            }
            1 => {
                let squish = 1.2 - t * 0.8;
                let squash_x = squish.max(0.6) * 1.3;
                let squash_y = (2.0 - squish).min(1.4) * 0.5;

                commands.entity(entity).insert(Transform {
                    translation: Vec3::new(transform.translation.x, transform.translation.y, Z_DEAD),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(squash_x, squash_y, 1.0),
                });

                if t > DEATH_COLLAPSE_DURATION {
                    commands.entity(entity).remove::<DeathAnimation>();
                    commands.entity(entity).insert(Transform {
                        translation: Vec3::new(transform.translation.x, transform.translation.y, Z_DEAD),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(1.0, 0.4, 1.0),
                    });
                }
            }
            _ => {}
        }
    }

    for (child_entity, world_pos) in balls_to_magnetize {
        commands.entity(child_entity).insert((
            MagnetizedBall,
            DespawnTimer(PARTICLE_LIFETIME),
        ));
        commands.entity(child_entity).remove_parent_in_place();
        commands.entity(child_entity).insert(Transform::from_xyz(world_pos.x, world_pos.y, world_pos.z));
    }
}
