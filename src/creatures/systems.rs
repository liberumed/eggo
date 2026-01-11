use bevy::prelude::*;

use crate::constants::*;
use crate::core::{Dead, DeathAnimation, DespawnTimer, Loot};
use crate::effects::{Hitstop, MagnetizedBall, ResourceBall};
use crate::player::{Player, SpriteAnimation};
use crate::player::Stats;
use crate::core::CharacterAssets;
use crate::state_machine::StateMachine;
use super::{Creature, CreatureAnimation, CreatureState, Goblin};

pub fn animate_creatures(
    time: Res<Time>,
    hitstop: Res<Hitstop>,
    mut query: Query<(&mut Transform, &CreatureAnimation), (With<Creature>, Without<Dead>)>,
) {
    if hitstop.is_active() {
        return;
    }

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

pub fn apply_collision_push(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Dead>, Without<Creature>)>,
    mut creatures_query: Query<(Entity, &mut Transform), (With<Creature>, Without<Dead>, Without<Player>)>,
) {
    let dt = time.delta_secs();

    let creature_positions: Vec<(Entity, Vec2)> = creatures_query
        .iter()
        .map(|(e, t)| (e, Vec2::new(t.translation.x, t.translation.y)))
        .collect();

    let player_pos = player_query
        .single()
        .map(|t| Vec2::new(t.translation.x, t.translation.y))
        .ok();

    for (entity, mut transform) in &mut creatures_query {
        let creature_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let mut push = Vec2::ZERO;

        if let Some(player_pos) = player_pos {
            let diff = creature_pos - player_pos;
            let dist = diff.length();

            if dist < PUSH_RADIUS && dist > 0.1 {
                let push_dir = diff.normalize();
                let overlap = (PUSH_RADIUS - dist) / PUSH_RADIUS;
                push += push_dir * overlap * PUSH_STRENGTH * dt;
            }
        }

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

        transform.translation.x += push.x;
        transform.translation.y += push.y;
    }
}

/// Updates goblin sprite animations based on state and movement
pub fn update_goblin_sprite_animation(
    player_query: Query<&Transform, (With<Player>, Without<Goblin>)>,
    mut goblin_query: Query<(&Transform, &mut SpriteAnimation, &StateMachine<CreatureState>), (With<Goblin>, Without<Dead>)>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();

    for (transform, mut sprite_anim, state_machine) in &mut goblin_query {
        let goblin_pos = transform.translation.truncate();
        let dir_to_player = player_pos - goblin_pos;

        match state_machine.current() {
            CreatureState::Attack(_) => {
                sprite_anim.set_animation("attack");
                sprite_anim.speed = 1.0;
                // Face toward player during attack (flip when player is to the left)
                sprite_anim.flip_x = dir_to_player.x < 0.0;
            }
            CreatureState::Chase | CreatureState::Cooldown => {
                // Use directional walk animations based on direction to player
                // (Cooldown uses same animation - creature still moves/follows)
                let vx = dir_to_player.x.abs();
                let vy = dir_to_player.y;

                let new_animation = if vy > vx {
                    "walk_up"
                } else if vy < -vx {
                    "walk_down"
                } else {
                    "walk"
                };

                sprite_anim.set_animation(new_animation);
                sprite_anim.speed = 1.0;
                sprite_anim.flip_x = dir_to_player.x < 0.0;
            }
            CreatureState::Idle | CreatureState::Stunned => {
                sprite_anim.set_animation("idle");
                sprite_anim.speed = 0.5;
            }
            CreatureState::Dying | CreatureState::Dead => {
                // Keep last animation frame when dying/dead
            }
        }
    }
}
