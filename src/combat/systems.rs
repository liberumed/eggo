use bevy::prelude::*;
use rand::Rng;

use crate::constants::*;
use crate::core::{ellipse_push, Blocking, Dead, DeathAnimation, GameAction, Health, HitCollider, InputBindings, Knockback, StaticCollider, Stunned};
use crate::creatures::{Creature, Hostile};
use crate::player::{Dashing, Player, PlayerAttackState};
use crate::props::{CrateSprite, Destructible, Prop, PropRegistry, PropType};
use super::{weapon_catalog, AttackType, CreatureRangeIndicator, Drawn, Fist, PlayerRangeIndicator, PlayerWeapon, Weapon, WeaponRangeIndicator, WeaponSwing, WeaponVisualMesh};
use crate::effects::{BloodParticle, HitHighlight, Hitstop, ScreenShake, TargetOutline};
use crate::core::CharacterAssets;
use crate::creatures::spawn_creature_range_indicator;
use super::hit_detection::{HitCone, angle_to_direction};
use super::mesh::create_weapon_arc;

pub fn toggle_weapon(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    weapon_query: Query<(Entity, &Weapon, Option<&Drawn>), With<PlayerWeapon>>,
) {
    if bindings.just_pressed(GameAction::ToggleWeapon, &keyboard, &mouse) {
        for (entity, weapon, drawn) in &weapon_query {
            if drawn.is_some() {
                commands.entity(entity).remove::<Drawn>();
                commands.entity(entity).insert(Visibility::Hidden);
            } else {
                commands.entity(entity).insert(Drawn);
                // Smash weapons don't show mesh (use sprite animation)
                if weapon.attack_type != AttackType::Smash {
                    commands.entity(entity).insert(Visibility::Inherited);
                }
            }
        }
    }
}

pub fn handle_block(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    player_query: Query<Entity, (With<Player>, Without<Dead>, Without<DeathAnimation>)>,
    weapon_query: Query<Option<&Drawn>, With<PlayerWeapon>>,
) {
    let Ok(player_entity) = player_query.single() else { return };
    let Ok(drawn) = weapon_query.single() else { return };

    if drawn.is_none() { return; }

    if bindings.pressed(GameAction::Block, &keyboard, &mouse) {
        commands.entity(player_entity).insert(Blocking);
    } else {
        commands.entity(player_entity).remove::<Blocking>();
    }
}

/// Starts weapon swing animation on attack input (damage applied later by apply_player_delayed_hits)
pub fn player_attack(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Dead>, Without<DeathAnimation>)>,
    weapon_query: Query<(Entity, &Transform, &Weapon, Option<&Drawn>), With<PlayerWeapon>>,
    swing_query: Query<&WeaponSwing, With<PlayerWeapon>>,
    attack_state_query: Query<&PlayerAttackState, With<Player>>,
) {
    if !bindings.just_pressed(GameAction::Attack, &keyboard, &mouse) {
        return;
    }

    // Already swinging (mesh animation) or attacking (sprite animation)
    if swing_query.iter().next().is_some() || attack_state_query.iter().next().is_some() {
        return;
    }

    let Ok((weapon_entity, weapon_transform, weapon, drawn)) = weapon_query.single() else { return };

    // First click draws weapon
    if drawn.is_none() {
        commands.entity(weapon_entity).insert(Drawn);
        // Smash weapons don't show mesh (use sprite animation)
        if weapon.attack_type != AttackType::Smash {
            commands.entity(weapon_entity).insert(Visibility::Inherited);
        }
        return;
    }

    let duration = weapon.swing_duration();

    // Smash weapons: use sprite animation
    if weapon.attack_type == AttackType::Smash {
        let Ok((player_entity, player_transform)) = player_query.single() else { return };

        // Determine direction from mouse position (left or right of player)
        let facing_right = if let Some(world_pos) = get_cursor_world_pos(&windows, &camera_query) {
            world_pos.x >= player_transform.translation.x
        } else {
            true // Default to right if no cursor
        };

        // Add attack state (locks direction for duration)
        commands.entity(player_entity).insert(PlayerAttackState {
            facing_right,
            timer: 0.0,
            duration,
            hit_applied: false,
        });
    } else {
        // Slash/Stab weapons: use mesh swing animation
        let (_, angle) = weapon_transform.rotation.to_axis_angle();
        let base_angle = if weapon_transform.rotation.z < 0.0 { -angle } else { angle };

        commands.entity(weapon_entity).insert(WeaponSwing {
            timer: 0.0,
            duration,
            base_angle: Some(base_angle),
            attack_type: weapon.attack_type,
            hit_delay: duration * ATTACK_HIT_DELAY_PERCENT,
            hit_applied: false,
        });
    }
}

/// Helper to get cursor world position
fn get_cursor_world_pos(
    windows: &Query<&Window>,
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = camera_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}

/// Applies damage when weapon swing reaches hit_delay (allows aiming during wind-up)
pub fn apply_player_delayed_hits(
    mut commands: Commands,
    mut hitstop: ResMut<Hitstop>,
    mut screen_shake: ResMut<ScreenShake>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Dead>, Without<DeathAnimation>)>,
    mut weapon_query: Query<(&Transform, &Weapon, &mut WeaponSwing), With<PlayerWeapon>>,
    mut creatures_query: Query<(Entity, &Transform, &mut Health, Option<&Hostile>, Option<&HitCollider>), (With<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    assets: Res<CharacterAssets>,
) {
    let Ok((weapon_transform, weapon, mut swing)) = weapon_query.single_mut() else { return };

    // Check if we've reached hit_delay and haven't applied hit yet
    if swing.timer < swing.hit_delay || swing.hit_applied {
        return;
    }
    swing.hit_applied = true;

    let Ok((player_entity, player_transform)) = player_query.single() else { return };

    // Attack origin: weapon position (same as debug cone)
    let weapon_offset = weapon_transform.translation;
    let attack_origin = Vec2::new(
        player_transform.translation.x + weapon_offset.x,
        player_transform.translation.y + weapon_offset.y,
    );

    // Attack direction from stored angle or current weapon rotation
    let attack_dir = if let Some(base_angle) = swing.base_angle {
        angle_to_direction(base_angle)
    } else {
        let (_, angle) = weapon_transform.rotation.to_axis_angle();
        let current_angle = if weapon_transform.rotation.z < 0.0 { -angle } else { angle };
        angle_to_direction(current_angle)
    };

    // Create hit cone (precomputes trig once)
    let hit_cone = HitCone::new(attack_origin, attack_dir, weapon.range(), weapon.cone_angle());

    let mut rng = rand::rng();
    let mut hit_any = false;

    for (entity, creature_transform, mut health, hostile, hit_collider) in &mut creatures_query {
        let creature_pos = creature_transform.translation.truncate();
        let hit_radius = hit_collider.map(|h| h.radius_x.max(h.radius_y)).unwrap_or(0.0);

        if hit_cone.hits(creature_pos, hit_radius) {
            hit_any = true;
            health.0 -= weapon.damage;

            // Knockback direction: from attack origin toward creature
            let knockback_dir = (creature_pos - hit_cone.origin).normalize_or_zero();
            weapon.apply_on_hit(&mut commands, entity, knockback_dir);

            // Add hit highlight (red flash)
            commands.entity(entity).insert(HitHighlight {
                timer: 0.0,
                duration: HIT_HIGHLIGHT_DURATION,
                original_material: None,
            });

            let particle_count = if health.0 <= 0 { 25 } else { 12 };
            for i in 0..particle_count {
                let spread = rng.random_range(-0.8..0.8);
                let speed = rng.random_range(80.0..200.0);
                let angle = attack_dir.y.atan2(attack_dir.x) + spread;
                let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);

                let is_splat = i % 3 == 0;
                let (mesh, material) = if is_splat {
                    (assets.blood_splat_mesh.clone(), assets.blood_splat_material.clone())
                } else {
                    (assets.blood_droplet_mesh.clone(), assets.blood_droplet_material.clone())
                };

                let offset = Vec2::new(
                    rng.random_range(-5.0..5.0),
                    rng.random_range(-5.0..5.0),
                );

                commands.spawn((
                    BloodParticle {
                        velocity: vel,
                        lifetime: rng.random_range(0.4..1.2),
                    },
                    Mesh2d(mesh),
                    MeshMaterial2d(material),
                    Transform::from_xyz(creature_pos.x + offset.x, creature_pos.y + offset.y, Z_BLOOD)
                        .with_rotation(Quat::from_rotation_z(rng.random_range(0.0..std::f32::consts::TAU))),
                ));
            }

            if health.0 <= 0 {
                commands.entity(entity).insert(DeathAnimation {
                    timer: 0.0,
                    stage: 0,
                });
            } else if hostile.is_none() {
                // Make non-hostile creature become hostile when hit
                commands.entity(entity).insert(Hostile { speed: PROVOKED_SPEED });

                // Spawn a fist for the newly hostile creature
                let fist_weapon = weapon_catalog::fist(&mut meshes, &mut materials);
                let fist_visual = fist_weapon.visual.clone();
                let arc_mesh = create_weapon_arc(&mut meshes, &fist_weapon);
                let fist_entity = commands.spawn((
                    Fist,
                    fist_weapon,
                    Transform::from_xyz(0.0, 0.0, Z_WEAPON),
                    Visibility::default(),
                )).with_children(|fist_holder| {
                    fist_holder.spawn((
                        WeaponVisualMesh,
                        Mesh2d(fist_visual.mesh),
                        MeshMaterial2d(fist_visual.material),
                        Transform::from_xyz(fist_visual.offset, 0.0, 0.0),
                    ));
                }).id();
                commands.entity(entity).add_child(fist_entity);

                // Range indicator as independent entity
                spawn_creature_range_indicator(
                    &mut commands,
                    entity,
                    arc_mesh,
                    assets.range_indicator_material.clone(),
                    creature_transform.translation,
                );
            }
        }
    }

    // Apply recoil and game feel effects when hitting
    if hit_any {
        let recoil_force = weapon.knockback_force() * 0.15;
        commands.entity(player_entity).insert(Knockback {
            velocity: -attack_dir * recoil_force,
            timer: 0.0,
        });

        // Trigger hitstop and screen shake
        hitstop.trigger(HITSTOP_DURATION);
        screen_shake.trigger(SCREEN_SHAKE_INTENSITY, SCREEN_SHAKE_DURATION);
    }
}

/// Updates sprite-based attack state, applies hit at 50% of duration
pub fn update_player_attack_state(
    mut commands: Commands,
    time: Res<Time>,
    mut hitstop: ResMut<Hitstop>,
    mut screen_shake: ResMut<ScreenShake>,
    prop_registry: Res<PropRegistry>,
    mut player_query: Query<(Entity, &Transform, &mut PlayerAttackState), With<Player>>,
    weapon_query: Query<(Entity, &Weapon), With<PlayerWeapon>>,
    mut creatures_query: Query<(Entity, &Transform, &mut Health, Option<&Hostile>, Option<&HitCollider>), (With<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    mut props_query: Query<(Entity, &Transform, &Prop, &mut Destructible, Option<&mut CrateSprite>, Option<&mut Sprite>), Without<Creature>>,
    assets: Res<CharacterAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok((player_entity, player_transform, mut attack_state)) = player_query.single_mut() else { return };

    // Freeze during hitstop
    if !hitstop.is_active() {
        attack_state.timer += time.delta_secs();
    }

    // Apply hit at 50% of animation
    let hit_time = attack_state.duration * ATTACK_HIT_DELAY_PERCENT;
    if attack_state.timer >= hit_time && !attack_state.hit_applied {
        attack_state.hit_applied = true;

        let Ok((_, weapon)) = weapon_query.single() else { return };

        // Attack direction based on locked facing
        let attack_dir = if attack_state.facing_right {
            Vec2::X
        } else {
            Vec2::NEG_X
        };

        // Attack origin: player position
        let attack_origin = player_transform.translation.truncate();

        // Create hit cone
        let hit_cone = HitCone::new(attack_origin, attack_dir, weapon.range(), weapon.cone_angle());

        let mut rng = rand::rng();
        let mut hit_any = false;

        for (entity, creature_transform, mut health, hostile, hit_collider) in &mut creatures_query {
            let creature_pos = creature_transform.translation.truncate();
            let hit_radius = hit_collider.map(|h| h.radius_x.max(h.radius_y)).unwrap_or(0.0);

            if hit_cone.hits(creature_pos, hit_radius) {
                hit_any = true;
                health.0 -= weapon.damage;

                // Knockback direction: from attack origin toward creature
                let knockback_dir = (creature_pos - hit_cone.origin).normalize_or_zero();
                weapon.apply_on_hit(&mut commands, entity, knockback_dir);

                // Add hit highlight (red flash)
                commands.entity(entity).insert(HitHighlight {
                    timer: 0.0,
                    duration: HIT_HIGHLIGHT_DURATION,
                    original_material: None,
                });

                let particle_count = if health.0 <= 0 { 25 } else { 12 };
                for i in 0..particle_count {
                    let spread = rng.random_range(-0.8..0.8);
                    let speed = rng.random_range(80.0..200.0);
                    let angle = attack_dir.y.atan2(attack_dir.x) + spread;
                    let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);

                    let is_splat = i % 3 == 0;
                    let (mesh, material) = if is_splat {
                        (assets.blood_splat_mesh.clone(), assets.blood_splat_material.clone())
                    } else {
                        (assets.blood_droplet_mesh.clone(), assets.blood_droplet_material.clone())
                    };

                    let offset = Vec2::new(
                        rng.random_range(-5.0..5.0),
                        rng.random_range(-5.0..5.0),
                    );

                    commands.spawn((
                        BloodParticle {
                            velocity: vel,
                            lifetime: rng.random_range(0.4..1.2),
                        },
                        Mesh2d(mesh),
                        MeshMaterial2d(material),
                        Transform::from_xyz(creature_pos.x + offset.x, creature_pos.y + offset.y, Z_BLOOD)
                            .with_rotation(Quat::from_rotation_z(rng.random_range(0.0..std::f32::consts::TAU))),
                    ));
                }

                if health.0 <= 0 {
                    commands.entity(entity).insert(DeathAnimation {
                        timer: 0.0,
                        stage: 0,
                    });
                } else if hostile.is_none() {
                    // Make non-hostile creature become hostile when hit
                    commands.entity(entity).insert(Hostile { speed: PROVOKED_SPEED });

                    // Spawn a fist for the newly hostile creature
                    let fist_weapon = weapon_catalog::fist(&mut meshes, &mut materials);
                    let fist_visual = fist_weapon.visual.clone();
                    let arc_mesh = create_weapon_arc(&mut meshes, &fist_weapon);
                    let fist_entity = commands.spawn((
                        Fist,
                        fist_weapon,
                        Transform::from_xyz(0.0, 0.0, Z_WEAPON),
                        Visibility::default(),
                    )).with_children(|fist_holder| {
                        fist_holder.spawn((
                            WeaponVisualMesh,
                            Mesh2d(fist_visual.mesh),
                            MeshMaterial2d(fist_visual.material),
                            Transform::from_xyz(fist_visual.offset, 0.0, 0.0),
                        ));
                    }).id();
                    commands.entity(entity).add_child(fist_entity);

                    // Range indicator as independent entity
                    spawn_creature_range_indicator(
                        &mut commands,
                        entity,
                        arc_mesh,
                        assets.range_indicator_material.clone(),
                        creature_transform.translation,
                    );
                }
            }
        }

        // Check destructible props (crates, barrels)
        for (entity, prop_transform, prop, mut destructible, crate_sprite, sprite) in &mut props_query {
            let prop_pos = prop_transform.translation.truncate();

            // Get hit_radius from prop registry
            let Some(definition) = prop_registry.get(prop.prop_type) else { continue };
            let Some(hit_radius) = definition.hit_radius else { continue };

            if hit_cone.hits(prop_pos, hit_radius) {
                hit_any = true;
                destructible.health -= weapon.damage;

                if destructible.health <= 0 {
                    // Destroy the crate
                    commands.entity(entity).despawn();
                } else if prop.prop_type == PropType::Crate {
                    // Show damaged sprite for crate
                    if let (Some(mut crate_state), Some(mut sprite)) = (crate_sprite, sprite) {
                        if !crate_state.damaged {
                            crate_state.damaged = true;
                            if let Some(atlas) = &mut sprite.texture_atlas {
                                atlas.index = 1;  // Damaged frame
                            }
                        }
                    }
                }
            }
        }

        // Apply recoil and game feel effects when hitting
        if hit_any {
            let recoil_force = weapon.knockback_force() * 0.15;
            commands.entity(player_entity).insert(Knockback {
                velocity: -attack_dir * recoil_force,
                timer: 0.0,
            });

            // Trigger hitstop and screen shake
            hitstop.trigger(HITSTOP_DURATION);
            screen_shake.trigger(SCREEN_SHAKE_INTENSITY, SCREEN_SHAKE_DURATION);
        }
    }

    // End attack when duration complete
    if attack_state.timer >= attack_state.duration {
        commands.entity(player_entity).remove::<PlayerAttackState>();
    }
}

pub fn aim_weapon(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut weapon_query: Query<(&mut Transform, &Weapon), (With<PlayerWeapon>, Without<Player>, Without<WeaponSwing>, Without<TargetOutline>)>,
    mut outline_query: Query<&mut Visibility, With<TargetOutline>>,
    blocking_query: Query<&Blocking>,
    attack_state_query: Query<&PlayerAttackState, With<Player>>,
) {
    // Don't aim weapon during sprite attack
    if attack_state_query.iter().next().is_some() {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok((player_entity, player_transform)) = player_query.single() else { return };
    let Ok((mut weapon_transform, weapon)) = weapon_query.single_mut() else { return };

    // Smash weapons don't show/aim mesh
    if weapon.attack_type == AttackType::Smash {
        return;
    }

    if let Ok(mut outline_visibility) = outline_query.single_mut() {
        *outline_visibility = Visibility::Hidden;
    }

    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);
    let dir = world_pos - player_pos;
    let angle = dir.y.atan2(dir.x);

    // Weapon offset from player center (at hand level for sprite)
    let weapon_offset = Vec2::new(WEAPON_OFFSET.0, WEAPON_OFFSET.1);

    let is_blocking = blocking_query.get(player_entity).is_ok();
    if is_blocking {
        // Blocking stance: pull weapon closer and tilt defensively
        let pull_back = -3.0;
        weapon_transform.rotation = Quat::from_rotation_z(angle + 0.4); // ~23° tilt
        weapon_transform.translation.x = weapon_offset.x + pull_back * angle.cos();
        weapon_transform.translation.y = weapon_offset.y + pull_back * angle.sin();
    } else {
        weapon_transform.rotation = Quat::from_rotation_z(angle);
        weapon_transform.translation.x = weapon_offset.x;
        weapon_transform.translation.y = weapon_offset.y;
    }
}

pub fn hostile_ai(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Creature>, Without<StaticCollider>)>,
    collider_query: Query<(&Transform, &StaticCollider), (Without<Player>, Without<Creature>)>,
    mut creature_queries: ParamSet<(
        Query<(Entity, &Transform), (With<Creature>, Without<Dead>, Without<StaticCollider>)>,
        Query<(Entity, &mut Transform, &Hostile), (Without<Dead>, Without<Player>, Without<Stunned>, Without<StaticCollider>)>,
    )>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    let creature_positions: Vec<(Entity, Vec2)> = creature_queries
        .p0()
        .iter()
        .map(|(e, t)| (e, Vec2::new(t.translation.x, t.translation.y)))
        .collect();

    let collider_data: Vec<(Vec2, Vec2)> = collider_query
        .iter()
        .map(|(t, c)| (Vec2::new(t.translation.x, t.translation.y + c.offset_y), Vec2::new(c.radius_x, c.radius_y)))
        .collect();

    let creature_collision_dist = COLLISION_RADIUS * 1.8;
    let player_collision_dist = COLLISION_RADIUS * 2.0;

    for (entity, mut transform, hostile) in creature_queries.p1().iter_mut() {
        let creature_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let distance = player_pos.distance(creature_pos);

        if distance < HOSTILE_SIGHT_RANGE && distance > player_collision_dist {
            let dir = (player_pos - creature_pos).normalize();
            let movement = dir * hostile.speed * time.delta_secs();
            let mut new_pos = creature_pos + movement;

            // Check collision with other creatures
            let blocked_by_creature = creature_positions.iter().any(|(other_entity, other_pos)| {
                *other_entity != entity && new_pos.distance(*other_pos) < creature_collision_dist
            });

            // Check collision with player - don't move if new position is too close
            let blocked_by_player = new_pos.distance(player_pos) < player_collision_dist;

            // Push-based ellipse collision with static colliders (at feet level)
            let creature_radius = Vec2::new(8.0, 5.0);
            let creature_offset_y = -11.0;  // Match shadow (ground footprint)
            for (collider_pos, collider_radius) in &collider_data {
                let creature_feet = Vec2::new(new_pos.x, new_pos.y + creature_offset_y);
                let push = ellipse_push(creature_feet, creature_radius, *collider_pos, *collider_radius);
                new_pos += push;
            }

            if !blocked_by_creature && !blocked_by_player {
                transform.translation.x = new_pos.x;
                transform.translation.y = new_pos.y;
            }
        }
    }
}

pub fn hostile_fist_aim(
    player_query: Query<&Transform, (With<Player>, Without<Creature>)>,
    hostile_query: Query<(&Transform, &Children), (With<Hostile>, Without<Dead>)>,
    mut fist_query: Query<&mut Transform, (With<Fist>, Without<Hostile>, Without<Player>, Without<WeaponSwing>)>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    for (creature_transform, children) in &hostile_query {
        let creature_pos = Vec2::new(creature_transform.translation.x, creature_transform.translation.y);

        for child in children.iter() {
            if let Ok(mut fist_transform) = fist_query.get_mut(child) {
                let dir = player_pos - creature_pos;
                let angle = dir.y.atan2(dir.x);
                fist_transform.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}

/// Starts creature swing animation when in range (damage applied later by apply_creature_delayed_hits)
pub fn hostile_attack(
    mut commands: Commands,
    player_query: Query<(&Transform, Option<&HitCollider>), (With<Player>, Without<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    hostile_query: Query<(&Transform, &Children), (With<Hostile>, Without<Dead>, Without<Stunned>)>,
    fist_query: Query<(Entity, &Weapon), (With<Fist>, Without<WeaponSwing>)>,
) {
    let Ok((player_transform, player_hit_collider)) = player_query.single() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);
    let player_hit_radius = player_hit_collider.map(|h| h.radius_x.max(h.radius_y)).unwrap_or(0.0);

    for (creature_transform, children) in &hostile_query {
        let creature_pos = Vec2::new(creature_transform.translation.x, creature_transform.translation.y);
        let distance = player_pos.distance(creature_pos);

        for child in children.iter() {
            if let Ok((fist_entity, weapon)) = fist_query.get(child) {
                if distance < weapon.range() + player_hit_radius {
                    // Start swing animation - hit applied later when hit_delay reached
                    let duration = weapon.swing_duration();
                    commands.entity(fist_entity).insert(WeaponSwing {
                        timer: 0.0,
                        duration,
                        base_angle: None,
                        attack_type: weapon.attack_type,
                        hit_delay: duration * ATTACK_HIT_DELAY_PERCENT,
                        hit_applied: false,
                    });
                }
            }
        }
    }
}

/// Applies creature damage when swing reaches hit_delay
pub fn apply_creature_delayed_hits(
    mut commands: Commands,
    mut hitstop: ResMut<Hitstop>,
    mut screen_shake: ResMut<ScreenShake>,
    mut player_query: Query<(Entity, &Transform, &mut Health, Option<&HitCollider>), (With<Player>, Without<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    hostile_query: Query<(Entity, &Transform, &Children), (With<Hostile>, Without<Dead>, Without<Stunned>)>,
    mut fist_query: Query<(&Weapon, &mut WeaponSwing), With<Fist>>,
    knockback_query: Query<&Knockback>,
    dashing_query: Query<&Dashing>,
    blocking_query: Query<&Blocking>,
    player_weapon_query: Query<(&Weapon, &Transform), With<PlayerWeapon>>,
) {
    let Ok((player_entity, player_transform, mut player_health, player_hit_collider)) = player_query.single_mut() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);
    let player_hit_radius = player_hit_collider.map(|h| h.radius_x.max(h.radius_y)).unwrap_or(0.0);

    // Invincible during dash (i-frames)
    if dashing_query.get(player_entity).is_ok() {
        return;
    }

    if knockback_query.get(player_entity).is_ok() {
        return;
    }

    let is_blocking = blocking_query.get(player_entity).is_ok();

    for (creature_entity, creature_transform, children) in &hostile_query {
        let creature_pos = creature_transform.translation.truncate();

        for child in children.iter() {
            if let Ok((weapon, mut swing)) = fist_query.get_mut(child) {
                // Check if we've reached hit_delay and haven't applied hit yet
                if swing.timer < swing.hit_delay || swing.hit_applied {
                    continue;
                }
                swing.hit_applied = true;

                // Cone attack toward player (same as player attacks)
                let attack_dir = (player_pos - creature_pos).normalize_or_zero();
                let hit_cone = HitCone::new(creature_pos, attack_dir, weapon.range(), weapon.cone_angle());

                if !hit_cone.hits(player_pos, player_hit_radius) {
                    continue;
                }

                // Check if block is effective (facing the attacker)
                let (damage_mult, kb_mult, blocked) = if is_blocking {
                    if let Ok((player_weapon, weapon_transform)) = player_weapon_query.single() {
                        let (_, angle) = weapon_transform.rotation.to_axis_angle();
                        let visual_angle = if weapon_transform.rotation.z < 0.0 { -angle } else { angle };
                        let facing_angle = visual_angle - 0.4;
                        let facing_dir = angle_to_direction(facing_angle);

                        // Optimized: avoid normalize by comparing dot > threshold * length
                        let to_attacker = creature_pos - player_pos;
                        let to_attacker_len = to_attacker.length();
                        let block_threshold = 0.5;
                        if to_attacker_len > 0.001 && facing_dir.dot(to_attacker) > block_threshold * to_attacker_len {
                            (1.0 - player_weapon.block_damage_reduction(), 1.0 - player_weapon.block_knockback_reduction(), true)
                        } else {
                            (1.0, 1.0, false)
                        }
                    } else {
                        (1.0, 1.0, false)
                    }
                } else {
                    (1.0, 1.0, false)
                };

                let final_damage = ((weapon.damage as f32) * damage_mult).floor() as i32;
                player_health.0 -= final_damage.max(0);

                let knockback_dir = (player_pos - creature_pos).normalize();
                commands.entity(player_entity).insert(Knockback {
                    velocity: knockback_dir * weapon.knockback_force() * kb_mult,
                    timer: 0.0,
                });

                // Knock back attacker when blocked
                if blocked {
                    commands.entity(creature_entity).insert(Knockback {
                        velocity: -knockback_dir * BLOCK_KNOCKBACK,
                        timer: 0.0,
                    });
                }

                // Trigger hitstop and screen shake (anime-style impact)
                hitstop.trigger(HITSTOP_DURATION);
                screen_shake.trigger(SCREEN_SHAKE_INTENSITY, SCREEN_SHAKE_DURATION);

                // Add hit highlight to player (red flash)
                commands.entity(player_entity).insert(HitHighlight {
                    timer: 0.0,
                    duration: HIT_HIGHLIGHT_DURATION,
                    original_material: None,
                });
                return;
            }
        }
    }
}

/// Sync player range indicator position and rotation with weapon aim
/// Computes aim angle directly from mouse position to avoid timing issues
pub fn sync_range_indicator(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    player_query: Query<(&Transform, &Children), With<Player>>,
    swinging_weapon_query: Query<&WeaponSwing, With<PlayerWeapon>>,
    mut indicator_query: Query<&mut Transform, (With<WeaponRangeIndicator>, With<PlayerRangeIndicator>, Without<Player>)>,
) {
    let Ok((player_transform, player_children)) = player_query.single() else { return };

    // Find indicator among player children
    let mut indicator_entity = None;
    for child in player_children.iter() {
        if indicator_query.get(child).is_ok() {
            indicator_entity = Some(child);
            break;
        }
    }
    let Some(indicator_entity) = indicator_entity else { return };
    let Ok(mut indicator_transform) = indicator_query.get_mut(indicator_entity) else { return };

    // Use base weapon offset
    let weapon_offset = Vec2::new(WEAPON_OFFSET.0, WEAPON_OFFSET.1);
    indicator_transform.translation.x = weapon_offset.x;
    indicator_transform.translation.y = weapon_offset.y;

    // If weapon is swinging, use base_angle (stored aim direction)
    if let Ok(swing) = swinging_weapon_query.single() {
        if let Some(base_angle) = swing.base_angle {
            indicator_transform.rotation = Quat::from_rotation_z(base_angle);
            return;
        }
    }

    // Compute aim angle directly from mouse position (same as aim_weapon)
    // This avoids timing issues with weapon tilt during blocking
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    let player_pos = player_transform.translation.truncate();
    let dir = world_pos - player_pos;
    let aim_angle = dir.y.atan2(dir.x);

    indicator_transform.rotation = Quat::from_rotation_z(aim_angle);
}

/// Sync creature range indicators position/rotation toward player (matches hit detection)
pub fn sync_creature_range_indicators(
    mut commands: Commands,
    player_query: Query<&Transform, (With<Player>, Without<Creature>, Without<CreatureRangeIndicator>)>,
    creature_query: Query<&Transform, (With<Creature>, With<Hostile>, Without<Dead>, Without<CreatureRangeIndicator>)>,
    mut indicator_query: Query<(Entity, &CreatureRangeIndicator, &mut Transform)>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();

    for (indicator_entity, link, mut indicator_transform) in &mut indicator_query {
        if let Ok(creature_transform) = creature_query.get(link.0) {
            let creature_pos = creature_transform.translation.truncate();
            // Sync position with creature
            indicator_transform.translation.x = creature_pos.x;
            indicator_transform.translation.y = creature_pos.y;
            // Calculate direction toward player (same as hit detection in apply_creature_delayed_hits)
            let dir = player_pos - creature_pos;
            let angle = dir.y.atan2(dir.x);
            indicator_transform.rotation = Quat::from_rotation_z(angle);
        } else {
            // Creature is dead or despawned, remove indicator
            commands.entity(indicator_entity).despawn();
        }
    }
}

/// Updates weapon visual mesh and range indicator when weapon stats change
pub fn update_weapon_visual(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    weapon_query: Query<(Entity, &Weapon, &ChildOf, Option<&Children>), (With<PlayerWeapon>, Changed<Weapon>)>,
    player_query: Query<&Children, With<Player>>,
    visual_mesh_query: Query<Entity, With<WeaponVisualMesh>>,
    indicator_query: Query<Entity, (With<WeaponRangeIndicator>, With<PlayerRangeIndicator>)>,
    assets: Res<CharacterAssets>,
) {
    for (weapon_entity, weapon, parent, children) in &weapon_query {
        // Despawn old weapon visual mesh
        if let Some(children) = children {
            for child in children.iter() {
                if visual_mesh_query.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }
        }

        // Despawn old indicator (sibling, child of player)
        if let Ok(player_children) = player_query.get(parent.0) {
            for child in player_children.iter() {
                if indicator_query.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }
        }

        // Spawn new visual from weapon data
        let visual = &weapon.visual;
        let visual_entity = commands
            .spawn((
                WeaponVisualMesh,
                Mesh2d(visual.mesh.clone()),
                MeshMaterial2d(visual.material.clone()),
                Transform::from_xyz(visual.offset, 0.0, 0.0),
            ))
            .id();
        commands.entity(weapon_entity).add_child(visual_entity);

        // Spawn new arc indicator as sibling (child of player)
        let arc_mesh = create_weapon_arc(&mut meshes, weapon);
        let weapon_offset = Vec2::new(WEAPON_OFFSET.0, WEAPON_OFFSET.1);
        let indicator_entity = commands
            .spawn((
                WeaponRangeIndicator,
                PlayerRangeIndicator,
                Mesh2d(arc_mesh),
                MeshMaterial2d(assets.range_indicator_material.clone()),
                Transform::from_xyz(weapon_offset.x, weapon_offset.y, Z_WEAPON + 0.1),
            ))
            .id();
        commands.entity(parent.0).add_child(indicator_entity);
    }
}
