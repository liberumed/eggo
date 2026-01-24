use bevy::prelude::*;

use crate::constants::*;
use crate::core::{Dead, DeathAnimation, DespawnTimer, ellipse_push, ellipses_overlap, Loot, WalkCollider};
use crate::effects::{Hitstop, MagnetizedBall, ResourceBall};
use crate::player::{HurtAnimation, Player, SpriteAnimation};
use crate::player::Stats;
use crate::core::CharacterAssets;
use crate::state_machine::{AttackPhase, StateMachine};
use super::{Creature, CreatureAnimation, CreatureState, Goblin, SpriteRendering};

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
    mut query: Query<(Entity, &Transform, &mut DeathAnimation, &Loot, &Children, Option<&SpriteRendering>), With<Creature>>,
    ball_query: Query<&Transform, With<ResourceBall>>,
) {
    let mut balls_to_magnetize: Vec<(Entity, Vec3)> = Vec::new();

    for (entity, transform, mut death, loot, children, sprite_rendering) in &mut query {
        death.timer += time.delta_secs();
        let t = death.timer;

        // Sprite-based creatures skip mesh squash effects
        let is_sprite_based = sprite_rendering.is_some();

        match death.stage {
            0 => {
                // Only apply shake/expand to mesh-based creatures
                if !is_sprite_based {
                    let shake = (t * 60.0).sin() * 0.15 * (1.0 - t * 2.0).max(0.0);
                    let expand = 1.0 + t * 0.5;

                    commands.entity(entity).insert(Transform {
                        translation: Vec3::new(transform.translation.x, transform.translation.y, Z_DEAD),
                        rotation: Quat::from_rotation_z(shake),
                        scale: Vec3::new(expand, expand * 0.9, 1.0),
                    });
                } else {
                    // Just update Z for sprite-based creatures
                    commands.entity(entity).insert(Transform {
                        translation: Vec3::new(transform.translation.x, transform.translation.y, Z_DEAD),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    });
                }

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

                    // Only apply dead material to mesh-based creatures
                    if is_sprite_based {
                        commands.entity(entity).insert((
                            Dead,
                            DespawnTimer(CORPSE_LIFETIME),
                        ));
                    } else {
                        commands.entity(entity).insert((
                            Dead,
                            DespawnTimer(CORPSE_LIFETIME),
                            MeshMaterial2d(assets.dead_material.clone()),
                        ));
                    }

                    if loot.philosophy { stats.philosophy += 1; }
                    if loot.nature_study { stats.nature_study += 1; }
                    if loot.wisdom { stats.wisdom += 1; }
                }
            }
            1 => {
                // Only apply squash to mesh-based creatures
                if !is_sprite_based {
                    let squish = 1.2 - t * 0.8;
                    let squash_x = squish.max(0.6) * 1.3;
                    let squash_y = (2.0 - squish).min(1.4) * 0.5;

                    commands.entity(entity).insert(Transform {
                        translation: Vec3::new(transform.translation.x, transform.translation.y, Z_DEAD),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(squash_x, squash_y, 1.0),
                    });
                }

                if t > DEATH_COLLAPSE_DURATION {
                    commands.entity(entity).remove::<DeathAnimation>();
                    if !is_sprite_based {
                        commands.entity(entity).insert(Transform {
                            translation: Vec3::new(transform.translation.x, transform.translation.y, Z_DEAD),
                            rotation: Quat::IDENTITY,
                            scale: Vec3::new(1.0, 0.4, 1.0),
                        });
                    }
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
    player_query: Query<(&Transform, &WalkCollider), (With<Player>, Without<Dead>, Without<Creature>)>,
    mut creatures_query: Query<(Entity, &mut Transform, &WalkCollider), (With<Creature>, Without<Dead>, Without<Player>)>,
) {
    let dt = time.delta_secs();

    // Collect creature data for creature-creature collision
    let creature_data: Vec<(Entity, Vec2, Vec2)> = creatures_query
        .iter()
        .map(|(e, t, w)| {
            let pos = Vec2::new(t.translation.x, t.translation.y + w.offset_y);
            let radius = Vec2::new(w.radius_x, w.radius_y);
            (e, pos, radius)
        })
        .collect();

    // Get player collision data
    let player_data = player_query
        .single()
        .map(|(t, w)| {
            let pos = Vec2::new(t.translation.x, t.translation.y + w.offset_y);
            let radius = Vec2::new(w.radius_x, w.radius_y);
            (pos, radius)
        })
        .ok();

    for (entity, mut transform, walk_collider) in &mut creatures_query {
        let creature_pos = Vec2::new(transform.translation.x, transform.translation.y + walk_collider.offset_y);
        let creature_radius = Vec2::new(walk_collider.radius_x, walk_collider.radius_y);
        let mut push = Vec2::ZERO;

        // Push away from player
        if let Some((player_pos, player_radius)) = player_data {
            if ellipses_overlap(creature_pos, creature_radius, player_pos, player_radius) {
                let raw_push = ellipse_push(creature_pos, creature_radius, player_pos, player_radius);
                push += raw_push * PUSH_STRENGTH * dt;
            }
        }

        // Push away from other creatures
        for (other_entity, other_pos, other_radius) in &creature_data {
            if *other_entity == entity {
                continue;
            }

            if ellipses_overlap(creature_pos, creature_radius, *other_pos, *other_radius) {
                let raw_push = ellipse_push(creature_pos, creature_radius, *other_pos, *other_radius);
                push += raw_push * PUSH_STRENGTH * 0.5 * dt;
            }
        }

        transform.translation.x += push.x;
        transform.translation.y += push.y;
    }
}

/// 4-way facing direction for goblin animations
enum GoblinFacing {
    Up,
    Down,
    Left,
    Right,
}

/// Determine 4-way facing direction from a direction vector
fn get_goblin_facing(dir: Vec2) -> GoblinFacing {
    if dir.length_squared() < 0.01 {
        return GoblinFacing::Down;
    }
    let abs_x = dir.x.abs();
    let abs_y = dir.y.abs();
    if abs_y > abs_x {
        if dir.y > 0.0 { GoblinFacing::Up } else { GoblinFacing::Down }
    } else {
        if dir.x > 0.0 { GoblinFacing::Right } else { GoblinFacing::Left }
    }
}

/// Updates goblin sprite animations based on state and movement
pub fn update_goblin_sprite_animation(
    player_query: Query<&Transform, (With<Player>, Without<Goblin>)>,
    mut goblin_query: Query<(&Transform, &mut SpriteAnimation, &StateMachine<CreatureState>, Option<&HurtAnimation>, Option<&DeathAnimation>), (With<Goblin>, Without<Dead>)>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();

    for (transform, mut sprite_anim, state_machine, hurt, death) in &mut goblin_query {
        let goblin_pos = transform.translation.truncate();
        let dir_to_player = player_pos - goblin_pos;

        // Determine facing direction based on direction to player
        let facing = get_goblin_facing(dir_to_player);

        // Death animation takes highest priority - show hurt sprite and freeze
        if death.is_some() {
            let hurt_anim = match facing {
                GoblinFacing::Up => "hurt_up",
                GoblinFacing::Down => "hurt_down",
                GoblinFacing::Left => "hurt_left",
                GoblinFacing::Right => "hurt_right",
            };
            sprite_anim.set_animation(hurt_anim);
            sprite_anim.speed = 0.0; // Pause animation
            sprite_anim.flip_x = false;
            continue;
        }

        // Hurt animation takes priority
        if hurt.is_some() {
            let hurt_anim = match facing {
                GoblinFacing::Up => "hurt_up",
                GoblinFacing::Down => "hurt_down",
                GoblinFacing::Left => "hurt_left",
                GoblinFacing::Right => "hurt_right",
            };
            sprite_anim.set_animation(hurt_anim);
            sprite_anim.speed = 1.0;
            sprite_anim.flip_x = false;
            continue;
        }

        match state_machine.current() {
            CreatureState::Attack(AttackPhase::Strike | AttackPhase::Recovery) => {
                let attack_anim = match facing {
                    GoblinFacing::Up => "att_up_1",
                    GoblinFacing::Down => "att_down_1",
                    GoblinFacing::Left => "att_left_1",
                    GoblinFacing::Right => "att_right_1",
                };
                sprite_anim.set_animation(attack_anim);
                sprite_anim.speed = 1.0;
                sprite_anim.flip_x = false;
            }
            CreatureState::Attack(AttackPhase::WindUp) => {
                let idle_anim = match facing {
                    GoblinFacing::Up => "idle_up",
                    GoblinFacing::Down => "idle_down",
                    GoblinFacing::Left => "idle_left",
                    GoblinFacing::Right => "idle_right",
                };
                sprite_anim.set_animation(idle_anim);
                sprite_anim.speed = 0.5;
                sprite_anim.flip_x = false;
            }
            CreatureState::Chase => {
                // Use directional walk animations
                let walk_anim = match facing {
                    GoblinFacing::Up => "walk_up",
                    GoblinFacing::Down => "walk_down",
                    GoblinFacing::Left => "walk_left",
                    GoblinFacing::Right => "walk_right",
                };
                sprite_anim.set_animation(walk_anim);
                sprite_anim.speed = 1.0;
                sprite_anim.flip_x = false;
            }
            CreatureState::Idle | CreatureState::Stunned | CreatureState::Cooldown => {
                // Use directional idle animations
                let idle_anim = match facing {
                    GoblinFacing::Up => "idle_up",
                    GoblinFacing::Down => "idle_down",
                    GoblinFacing::Left => "idle_left",
                    GoblinFacing::Right => "idle_right",
                };
                sprite_anim.set_animation(idle_anim);
                sprite_anim.speed = 0.5;
                sprite_anim.flip_x = false;
            }
            CreatureState::Dying | CreatureState::Dead => {
                // Show hurt animation and pause on death
                let hurt_anim = match facing {
                    GoblinFacing::Up => "hurt_up",
                    GoblinFacing::Down => "hurt_down",
                    GoblinFacing::Left => "hurt_left",
                    GoblinFacing::Right => "hurt_right",
                };
                sprite_anim.set_animation(hurt_anim);
                sprite_anim.speed = 0.0; // Pause animation
                sprite_anim.flip_x = false;
            }
        }
    }
}
