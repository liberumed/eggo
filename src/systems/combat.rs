use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::spawners::CharacterAssets;

pub fn toggle_weapon(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    weapon_query: Query<(Entity, &Visibility, Option<&Drawn>), With<PlayerWeapon>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for (entity, _, drawn) in &weapon_query {
            if drawn.is_some() {
                commands.entity(entity).remove::<Drawn>();
                commands.entity(entity).insert(Visibility::Hidden);
            } else {
                commands.entity(entity).insert(Drawn);
                commands.entity(entity).insert(Visibility::Inherited);
            }
        }
    }
}

pub fn handle_block(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    player_query: Query<Entity, (With<Player>, Without<Dead>, Without<DeathAnimation>)>,
    weapon_query: Query<Option<&Drawn>, With<PlayerWeapon>>,
) {
    let Ok(player_entity) = player_query.single() else { return };
    let Ok(drawn) = weapon_query.single() else { return };

    if drawn.is_none() { return; }

    if mouse.pressed(MouseButton::Right) {
        commands.entity(player_entity).insert(Blocking);
    } else {
        commands.entity(player_entity).remove::<Blocking>();
    }
}

pub fn player_attack(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, (With<Player>, Without<Dead>, Without<DeathAnimation>)>,
    weapon_query: Query<(Entity, &Transform, &Weapon, Option<&Drawn>), With<PlayerWeapon>>,
    swing_query: Query<&WeaponSwing, With<PlayerWeapon>>,
    mut creatures_query: Query<(Entity, &Transform, &mut Health, Option<&Hostile>), (With<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    assets: Res<CharacterAssets>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    if swing_query.iter().next().is_some() {
        return;
    }

    let Ok((weapon_entity, _, weapon, drawn)) = weapon_query.single() else { return };
    if drawn.is_none() {
        commands.entity(weapon_entity).insert(Drawn);
        commands.entity(weapon_entity).insert(Visibility::Inherited);
        return;
    }

    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    let attack_dir = if let Ok((weapon_entity, weapon_transform, weapon, _)) = weapon_query.single() {
        let (_, angle) = weapon_transform.rotation.to_axis_angle();
        let base_angle = if weapon_transform.rotation.z < 0.0 { -angle } else { angle };
        commands.entity(weapon_entity).insert(WeaponSwing {
            timer: 0.0,
            duration: weapon.swing_duration(),
            base_angle: Some(base_angle),
        });
        Vec2::new(base_angle.cos(), base_angle.sin())
    } else {
        Vec2::X
    };

    let cone_threshold = (weapon.cone_angle() / 2.0).cos();
    let mut rng = rand::rng();

    for (entity, creature_transform, mut health, hostile) in &mut creatures_query {
        let creature_pos = Vec2::new(creature_transform.translation.x, creature_transform.translation.y);
        let to_creature = creature_pos - player_pos;
        let distance = to_creature.length();

        let in_range = distance < weapon.range();
        let in_cone = distance > 0.0 && to_creature.normalize().dot(attack_dir) > cone_threshold;

        if in_range && in_cone {
            health.0 -= weapon.damage;
            commands.entity(entity).insert(Stunned(STUN_DURATION));

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
                let fist_entity = commands.spawn((
                    Fist,
                    catalog::fist(),
                    Transform::from_xyz(0.0, 0.0, Z_WEAPON),
                    Visibility::default(),
                )).with_children(|fist_holder| {
                    fist_holder.spawn((
                        Mesh2d(assets.fist_mesh.clone()),
                        MeshMaterial2d(assets.fist_material.clone()),
                        Transform::from_xyz(11.0, 0.0, 0.0),
                    ));
                }).id();

                commands.entity(entity).add_child(fist_entity);
            }
        }
    }
}

pub fn aim_weapon(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut weapon_query: Query<&mut Transform, (With<PlayerWeapon>, Without<Player>, Without<WeaponSwing>, Without<TargetOutline>)>,
    mut outline_query: Query<&mut Visibility, With<TargetOutline>>,
    blocking_query: Query<&Blocking>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok((player_entity, player_transform)) = player_query.single() else { return };
    let Ok(mut weapon_transform) = weapon_query.single_mut() else { return };

    if let Ok(mut outline_visibility) = outline_query.single_mut() {
        *outline_visibility = Visibility::Hidden;
    }

    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);
    let dir = world_pos - player_pos;
    let angle = dir.y.atan2(dir.x);

    let is_blocking = blocking_query.get(player_entity).is_ok();
    if is_blocking {
        // Blocking stance: pull weapon closer and tilt defensively
        let pull_back = -3.0;
        weapon_transform.rotation = Quat::from_rotation_z(angle + 0.4); // ~23° tilt
        weapon_transform.translation.x = pull_back * angle.cos();
        weapon_transform.translation.y = pull_back * angle.sin();
    } else {
        weapon_transform.rotation = Quat::from_rotation_z(angle);
        weapon_transform.translation.x = 0.0;
        weapon_transform.translation.y = 0.0;
    }
}

pub fn hostile_ai(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Creature>)>,
    mut creature_queries: ParamSet<(
        Query<(Entity, &Transform), (With<Creature>, Without<Dead>)>,
        Query<(Entity, &mut Transform, &Hostile), (Without<Dead>, Without<Player>, Without<Stunned>)>,
    )>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    let creature_positions: Vec<(Entity, Vec2)> = creature_queries
        .p0()
        .iter()
        .map(|(e, t)| (e, Vec2::new(t.translation.x, t.translation.y)))
        .collect();

    let creature_collision_dist = COLLISION_RADIUS * 1.8;
    let player_collision_dist = COLLISION_RADIUS * 2.0;

    for (entity, mut transform, hostile) in creature_queries.p1().iter_mut() {
        let creature_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let distance = player_pos.distance(creature_pos);

        if distance < HOSTILE_SIGHT_RANGE && distance > player_collision_dist {
            let dir = (player_pos - creature_pos).normalize();
            let movement = dir * hostile.speed * time.delta_secs();
            let new_pos = creature_pos + movement;

            // Check collision with other creatures
            let blocked_by_creature = creature_positions.iter().any(|(other_entity, other_pos)| {
                *other_entity != entity && new_pos.distance(*other_pos) < creature_collision_dist
            });

            // Check collision with player - don't move if new position is too close
            let blocked_by_player = new_pos.distance(player_pos) < player_collision_dist;

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

pub fn hostile_attack(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &mut Health), (With<Player>, Without<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    hostile_query: Query<(&Transform, &Children), (With<Hostile>, Without<Dead>, Without<Stunned>)>,
    fist_query: Query<(Entity, &Weapon), (With<Fist>, Without<WeaponSwing>)>,
    knockback_query: Query<&Knockback>,
    blocking_query: Query<&Blocking>,
    player_weapon_query: Query<(&Weapon, &Transform), With<PlayerWeapon>>,
) {
    let Ok((player_entity, player_transform, mut player_health)) = player_query.single_mut() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    if knockback_query.get(player_entity).is_ok() {
        return;
    }

    let is_blocking = blocking_query.get(player_entity).is_ok();

    for (creature_transform, children) in &hostile_query {
        let creature_pos = Vec2::new(creature_transform.translation.x, creature_transform.translation.y);
        let distance = player_pos.distance(creature_pos);

        for child in children.iter() {
            if let Ok((fist_entity, weapon)) = fist_query.get(child) {
                if distance < weapon.range() {
                    commands.entity(fist_entity).insert(WeaponSwing {
                        timer: 0.0,
                        duration: weapon.swing_duration(),
                        base_angle: None,
                    });

                    // Check if block is effective (facing the attacker)
                    let (damage_mult, kb_mult) = if is_blocking {
                        if let Ok((player_weapon, weapon_transform)) = player_weapon_query.single() {
                            // Get weapon facing direction from rotation
                            // Note: weapon is tilted 0.4 rad during block animation, so subtract that offset
                            let (_, angle) = weapon_transform.rotation.to_axis_angle();
                            let visual_angle = if weapon_transform.rotation.z < 0.0 { -angle } else { angle };
                            let facing_angle = visual_angle - 0.4;
                            let facing_dir = Vec2::new(facing_angle.cos(), facing_angle.sin());

                            // Direction from player to attacker
                            let to_attacker = (creature_pos - player_pos).normalize();

                            // Block works if facing toward the attacker (within ~120 degree arc)
                            let block_threshold = 0.5; // cos(60°) - blocks attacks within 60° of facing
                            if facing_dir.dot(to_attacker) > block_threshold {
                                (1.0 - player_weapon.block_damage_reduction(), 1.0 - player_weapon.block_knockback_reduction())
                            } else {
                                (1.0, 1.0) // Attack from behind - no block
                            }
                        } else {
                            (1.0, 1.0)
                        }
                    } else {
                        (1.0, 1.0)
                    };

                    let final_damage = ((weapon.damage as f32) * damage_mult).floor() as i32;
                    player_health.0 -= final_damage.max(0);

                    let knockback_dir = (player_pos - creature_pos).normalize();
                    commands.entity(player_entity).insert(Knockback {
                        velocity: knockback_dir * weapon.knockback() * kb_mult,
                        timer: 0.0,
                    });
                    return;
                }
            }
        }
    }
}
