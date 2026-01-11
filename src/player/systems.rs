use bevy::prelude::*;

use crate::constants::{
    CAMERA_ZOOM_SPEED, DEATH_COLLAPSE_DURATION, DEATH_EXPAND_DURATION, Z_DEAD,
};
use crate::core::{Blocking, Dead, DeathAnimation, GameConfig, WalkCollider, StaticCollider, ellipses_overlap, ellipse_push, Health, Knockback};
use crate::effects::{Hitstop, ScreenShake};
use crate::core::{GameAction, InputBindings};
use crate::core::CharacterAssets;
use crate::state_machine::{AttackPhase, RequestTransition, StateMachine};
use super::{
    Player, PlayerAnimation, DashCooldown, Sprinting, PhaseThrough,
    SpriteAnimation, PlayerSpriteSheet, PlayerState, CameraState,
    PlayerDashing, PlayerAttacking, PlayerSmashAttack,
};
use crate::inventory::AttackType;
use crate::inventory::weapons::{Weapon, PlayerWeapon, Drawn, WeaponSwing, Fist};
use crate::creatures::Creature;
use crate::levels::CurrentLevel;

pub fn move_player(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut transitions: MessageWriter<RequestTransition<PlayerState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    hitstop: Res<Hitstop>,
    time: Res<Time>,
    mut player_query: Query<(Entity, &mut Transform, &mut PlayerAnimation, &WalkCollider, &StateMachine<PlayerState>, Option<&mut Sprinting>, Option<&PhaseThrough>), (With<Player>, Without<Dead>, Without<DeathAnimation>)>,
    creatures_query: Query<(&Transform, &WalkCollider, Option<&Dead>), (With<Creature>, Without<Player>)>,
    colliders_query: Query<(&Transform, &StaticCollider), Without<Player>>,
    blocking_query: Query<&Blocking>,
) {
    if hitstop.is_active() {
        return;
    }

    let dt = time.delta_secs();

    for (entity, mut transform, mut anim, walk_collider, state, sprinting, phase_through) in &mut player_query {
        let current_state = state.current();

        let can_move = matches!(current_state, PlayerState::Idle | PlayerState::Moving);
        if !can_move {
            continue;
        }

        let mut input_dir = Vec2::ZERO;
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

        let player_radius = Vec2::new(walk_collider.radius_x, walk_collider.radius_y);
        let player_offset_y = walk_collider.offset_y;
        let is_blocking = blocking_query.get(entity).is_ok();
        let is_sprinting = bindings.pressed(GameAction::Sprint, &keyboard, &mouse) && input_dir != Vec2::ZERO;
        let is_phasing = phase_through.is_some();

        let sprint_multiplier = if is_sprinting {
            let sprint_duration = if let Some(mut sprint) = sprinting {
                sprint.duration += dt;
                sprint.duration
            } else {
                commands.entity(entity).insert(Sprinting { duration: 0.0 });
                0.0
            };
            let t = (sprint_duration / config.sprint_ramp_time).min(1.0);
            config.sprint_min_multiplier + t * (config.sprint_max_multiplier - config.sprint_min_multiplier)
        } else {
            commands.entity(entity).remove::<Sprinting>();
            1.0
        };

        let speed = if is_blocking {
            config.player_speed * 0.4
        } else {
            config.player_speed * sprint_multiplier
        };

        if input_dir != Vec2::ZERO {
            let target_velocity = input_dir.normalize() * speed;
            let diff = target_velocity - anim.velocity;
            let current_speed = anim.velocity.length();

            let is_decelerating = current_speed > speed;
            let accel_amount = if is_decelerating && current_speed > config.player_speed * 1.1 {
                config.sprint_momentum_friction
            } else {
                config.player_acceleration
            };

            let accel = diff.normalize_or_zero() * accel_amount * dt;
            if accel.length() > diff.length() {
                anim.velocity = target_velocity;
            } else {
                anim.velocity += accel;
            }

            if *current_state == PlayerState::Idle {
                transitions.write(RequestTransition::new(entity, PlayerState::Moving));
            }
        } else if anim.velocity != Vec2::ZERO {
            let current_speed = anim.velocity.length();
            let friction_amount = if current_speed > config.player_speed * 1.1 {
                config.sprint_momentum_friction
            } else {
                config.player_friction
            };
            let friction = anim.velocity.normalize() * friction_amount * dt;
            if friction.length() >= anim.velocity.length() {
                anim.velocity = Vec2::ZERO;
                if *current_state == PlayerState::Moving {
                    transitions.write(RequestTransition::new(entity, PlayerState::Idle));
                }
            } else {
                anim.velocity -= friction;
            }
        } else if *current_state == PlayerState::Moving {
            transitions.write(RequestTransition::new(entity, PlayerState::Idle));
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

                let test_x = Vec2::new(new_pos.x, transform.translation.y + player_offset_y);
                if ellipses_overlap(test_x, player_radius, creature_pos, creature_radius) {
                    blocked_x = true;
                }

                let test_y = Vec2::new(transform.translation.x, new_pos.y + player_offset_y);
                if ellipses_overlap(test_y, player_radius, creature_pos, creature_radius) {
                    blocked_y = true;
                }
            }
        }

        let mut static_push = Vec2::ZERO;
        for (collider_transform, collider) in &colliders_query {
            let collider_pos = Vec2::new(
                collider_transform.translation.x + collider.offset_x,
                collider_transform.translation.y + collider.offset_y,
            );
            let collider_radius = Vec2::new(collider.radius_x, collider.radius_y);

            let player_pos = Vec2::new(new_pos.x, new_pos.y + player_offset_y);
            let push = ellipse_push(player_pos, player_radius, collider_pos, collider_radius);
            static_push += push;
        }

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

pub fn apply_dash_state(
    config: Res<GameConfig>,
    mut transitions: MessageWriter<RequestTransition<PlayerState>>,
    time: Res<Time>,
    hitstop: Res<Hitstop>,
    mut query: Query<(Entity, &mut Transform, &mut PlayerDashing, &mut PlayerAnimation, &StateMachine<PlayerState>), With<Player>>,
) {
    if hitstop.is_active() {
        return;
    }

    let dt = time.delta_secs();

    for (entity, mut transform, mut dash, mut anim, state) in &mut query {
        if *state.current() != PlayerState::Dashing {
            continue;
        }

        dash.timer -= dt;

        let movement = dash.direction * config.dash_speed * dt;
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;

        anim.velocity = dash.direction * config.player_speed;

        if dash.timer <= 0.0 {
            transitions.write(RequestTransition::new(entity, PlayerState::Moving));
        }
    }
}

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

pub fn animate_weapon_swing(
    mut commands: Commands,
    time: Res<Time>,
    hitstop: Res<Hitstop>,
    mut query: Query<(Entity, &mut Transform, &mut WeaponSwing, Option<&Fist>)>,
) {
    if hitstop.is_active() {
        return;
    }

    for (entity, mut transform, mut swing, is_fist) in &mut query {
        swing.timer += time.delta_secs();
        let t = swing.timer;

        if t < swing.duration {
            let progress = t / swing.duration;

            if let Some(base_angle) = swing.base_angle {
                match swing.attack_type {
                    AttackType::Smash => {
                        let swing_arc = 1.5;
                        let (scale, rotation_offset) = if progress < 0.15 {
                            let p = progress / 0.15;
                            (1.0, -swing_arc * 0.6 * p)
                        } else if progress < 0.45 {
                            let p = (progress - 0.15) / 0.3;
                            (1.0 + p * 0.15, -swing_arc * 0.6 + swing_arc * 1.0 * p)
                        } else {
                            let p = (progress - 0.45) / 0.55;
                            (1.15 - p * 0.15, swing_arc * 0.4 - swing_arc * 0.4 * p)
                        };
                        transform.scale = Vec3::new(scale, 1.0, 1.0);
                        transform.rotation = Quat::from_rotation_z(base_angle + rotation_offset);
                    }
                    AttackType::Slash | AttackType::Stab => {
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
                    }
                }
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
            // Only auto-remove for player weapons (not Fist)
            // Creature fists are cleaned up by on_attack_exit via state machine
            if is_fist.is_none() {
                commands.entity(entity).remove::<WeaponSwing>();
            }
        }
    }
}

pub fn animate_player_death(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<CharacterAssets>,
    mut query: Query<(Entity, &Transform, &mut DeathAnimation), With<Player>>,
) {
    for (entity, transform, mut death) in &mut query {
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

                    commands.entity(entity).insert((
                        Dead,
                        MeshMaterial2d(assets.dead_material.clone()),
                    ));
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
}

pub fn advance_player_attack_phases(
    config: Res<GameConfig>,
    mut transitions: MessageWriter<RequestTransition<PlayerState>>,
    time: Res<Time>,
    hitstop: Res<Hitstop>,
    mut smash_query: Query<(Entity, &mut PlayerSmashAttack, &StateMachine<PlayerState>), With<Player>>,
    mut swing_query: Query<(&WeaponSwing, &ChildOf), With<PlayerWeapon>>,
    player_query: Query<(Entity, &StateMachine<PlayerState>), With<Player>>,
) {
    if hitstop.is_active() {
        return;
    }

    let dt = time.delta_secs();

    for (entity, mut smash, state) in &mut smash_query {
        let PlayerState::Attacking(current_phase) = state.current() else { continue };

        smash.timer += dt;

        match current_phase {
            AttackPhase::WindUp => {
                let hit_delay = smash.duration * config.attack_hit_delay_percent;
                if smash.timer >= hit_delay {
                    transitions.write(RequestTransition::new(
                        entity,
                        PlayerState::Attacking(AttackPhase::Strike),
                    ));
                }
            }
            AttackPhase::Strike => {
                if smash.hit_applied {
                    transitions.write(RequestTransition::new(
                        entity,
                        PlayerState::Attacking(AttackPhase::Recovery),
                    ));
                }
            }
            AttackPhase::Recovery => {
                if smash.timer >= smash.duration {
                    transitions.write(RequestTransition::new(entity, PlayerState::Idle));
                }
            }
        }
    }

    for (swing, parent) in &mut swing_query {
        let Ok((entity, state)) = player_query.get(parent.parent()) else { continue };
        let PlayerState::Attacking(current_phase) = state.current() else { continue };

        match current_phase {
            AttackPhase::WindUp => {
                if swing.timer >= swing.hit_delay {
                    transitions.write(RequestTransition::new(
                        entity,
                        PlayerState::Attacking(AttackPhase::Strike),
                    ));
                }
            }
            AttackPhase::Strike => {
                if swing.hit_applied {
                    transitions.write(RequestTransition::new(
                        entity,
                        PlayerState::Attacking(AttackPhase::Recovery),
                    ));
                }
            }
            AttackPhase::Recovery => {
                if swing.timer >= swing.duration {
                    transitions.write(RequestTransition::new(entity, PlayerState::Idle));
                }
            }
        }
    }
}

pub fn update_player_sprite_animation(
    config: Res<GameConfig>,
    mut query: Query<(&PlayerAnimation, &mut SpriteAnimation, &StateMachine<PlayerState>, Option<&PlayerAttacking>), With<Player>>,
    weapon_query: Query<(&Weapon, Option<&Drawn>), With<PlayerWeapon>>,
) {
    let has_smash_weapon = weapon_query
        .iter()
        .next()
        .map(|(w, drawn)| w.attack_type == AttackType::Smash && drawn.is_some())
        .unwrap_or(false);

    for (player_anim, mut sprite_anim, state, attacking) in &mut query {
        if matches!(state.current(), PlayerState::Attacking(_)) {
            if let Some(attack) = attacking {
                sprite_anim.set_animation("attack");
                sprite_anim.speed = 1.0;
                sprite_anim.flip_x = !attack.facing_right;
            }
            continue;
        }

        let velocity = player_anim.velocity.length();

        let (new_animation, anim_speed) = if velocity > config.player_speed * 1.2 {
            ("run", 1.0)
        } else if velocity > 0.1 {
            let vx = player_anim.velocity.x.abs();
            let vy = player_anim.velocity.y;
            if vy > vx {
                ("walk_up", 1.0)
            } else if vy < -vx {
                ("walk_down", 1.0)
            } else {
                ("walk", 1.0)
            }
        } else if has_smash_weapon {
            ("idle_stick", 1.0)
        } else {
            ("idle", 0.2)
        };

        sprite_anim.set_animation(new_animation);
        sprite_anim.speed = anim_speed;

        if player_anim.velocity.x.abs() > 0.1 {
            sprite_anim.flip_x = player_anim.velocity.x > 0.0;
        }
    }
}

pub fn animate_sprites(
    time: Res<Time>,
    sprite_sheet: Option<Res<PlayerSpriteSheet>>,
    mut query: Query<(&mut SpriteAnimation, &mut Sprite)>,
) {
    let Some(sprite_sheet) = sprite_sheet else { return };

    for (mut anim, mut sprite) in &mut query {
        if anim.animation_changed {
            if let Some(data) = sprite_sheet.animations.get(&anim.current_animation) {
                sprite.image = data.texture.clone();
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.layout = data.atlas_layout.clone();
                    atlas.index = data.start_index;
                }
                anim.timer.set_duration(std::time::Duration::from_millis(data.frame_duration_ms as u64));
            }
            anim.animation_changed = false;
        }

        let scaled_delta = time.delta().mul_f32(anim.speed);
        anim.timer.tick(scaled_delta);

        if anim.timer.just_finished() {
            if let Some(data) = sprite_sheet.animations.get(&anim.current_animation) {
                if data.looping {
                    anim.frame_index = (anim.frame_index + 1) % data.frame_count;
                } else if anim.frame_index < data.frame_count - 1 {
                    anim.frame_index += 1;
                }

                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = data.start_index + anim.frame_index;
                }
            }
        }

        sprite.flip_x = anim.flip_x;
    }
}

pub fn camera_follow(
    time: Res<Time>,
    player_query: Query<(&Transform, &PlayerAnimation), With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    mut camera_state: ResMut<CameraState>,
    screen_shake: Res<ScreenShake>,
    current_level: Res<CurrentLevel>,
) {
    let Ok((player_transform, player_anim)) = player_query.single() else { return };
    let Ok(mut camera_transform) = camera_query.single_mut() else { return };

    // Update zoom based on player velocity
    let is_moving = player_anim.velocity.length() > 1.0;
    camera_state.set_moving(is_moving);

    // Smooth lerp toward target zoom
    let dt = time.delta_secs();
    camera_state.current_scale = camera_state.current_scale
        + (camera_state.target_scale - camera_state.current_scale) * CAMERA_ZOOM_SPEED * dt;

    // Calculate base camera position from player
    let mut camera_x = player_transform.translation.x;
    let mut camera_y = player_transform.translation.y;

    // Clamp camera to level bounds if level is loaded
    if let Some(bounds) = current_level.bounds() {
        let half_view_x = 160.0 * camera_state.current_scale * 4.0;
        let half_view_y = 120.0 * camera_state.current_scale * 4.0;

        let min_x = bounds.min.x + half_view_x;
        let max_x = bounds.max.x - half_view_x;
        let min_y = bounds.min.y + half_view_y;
        let max_y = bounds.max.y - half_view_y;

        if min_x < max_x {
            camera_x = camera_x.clamp(min_x, max_x);
        } else {
            camera_x = bounds.center().x;
        }
        if min_y < max_y {
            camera_y = camera_y.clamp(min_y, max_y);
        } else {
            camera_y = bounds.center().y;
        }
    }

    // Apply position with shake
    let shake_offset = screen_shake.get_offset();
    camera_transform.translation.x = camera_x + shake_offset.x;
    camera_transform.translation.y = camera_y + shake_offset.y;

    // Apply zoom
    camera_transform.scale = Vec3::splat(camera_state.current_scale);
}
