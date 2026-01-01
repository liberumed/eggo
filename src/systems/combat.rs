use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::spawners::CharacterAssets;

pub fn toggle_weapon(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut knife_query: Query<&mut Visibility, With<Knife>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for mut visibility in &mut knife_query {
            *visibility = match *visibility {
                Visibility::Hidden => Visibility::Inherited,
                _ => Visibility::Hidden,
            };
        }
    }
}

pub fn knife_attack(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, (With<Player>, Without<Dead>)>,
    knife_query: Query<(Entity, &Transform, &Visibility), With<Knife>>,
    knife_swing_query: Query<&WeaponSwing, With<Knife>>,
    mut creatures_query: Query<(Entity, &Transform, &mut Health, Option<&Hostile>), (With<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    assets: Res<CharacterAssets>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    if knife_swing_query.iter().next().is_some() {
        return;
    }

    let Ok((knife_entity, _, visibility)) = knife_query.single() else { return };
    if *visibility == Visibility::Hidden {
        commands.entity(knife_entity).insert(Visibility::Inherited);
        return;
    }

    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    let knife_dir = if let Ok((knife_entity, knife_transform, _)) = knife_query.single() {
        let (_, angle) = knife_transform.rotation.to_axis_angle();
        let base_angle = if knife_transform.rotation.z < 0.0 { -angle } else { angle };
        commands.entity(knife_entity).insert(WeaponSwing { timer: 0.0, duration: KNIFE_SWING_DURATION, base_angle: Some(base_angle) });
        Vec2::new(base_angle.cos(), base_angle.sin())
    } else {
        Vec2::X
    };

    let mut rng = rand::rng();

    for (entity, creature_transform, mut health, hostile) in &mut creatures_query {
        let creature_pos = Vec2::new(creature_transform.translation.x, creature_transform.translation.y);
        let to_creature = creature_pos - player_pos;
        let distance = to_creature.length();

        let in_range = distance < KNIFE_RANGE;
        let in_cone = distance > 0.0 && to_creature.normalize().dot(knife_dir) > 0.5;

        if in_range && in_cone {
            health.0 -= 1;
            commands.entity(entity).insert(Stunned(STUN_DURATION));

            let particle_count = if health.0 <= 0 { 25 } else { 12 };
            for i in 0..particle_count {
                let spread = rng.random_range(-0.8..0.8);
                let speed = rng.random_range(80.0..200.0);
                let angle = knife_dir.y.atan2(knife_dir.x) + spread;
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

pub fn aim_knife(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    player_query: Query<&Transform, With<Player>>,
    mut knife_query: Query<&mut Transform, (With<Knife>, Without<Player>, Without<WeaponSwing>, Without<TargetOutline>)>,
    mut outline_query: Query<&mut Visibility, With<TargetOutline>>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(player_transform) = player_query.single() else { return };
    let Ok(mut knife_transform) = knife_query.single_mut() else { return };

    if let Ok(mut outline_visibility) = outline_query.single_mut() {
        *outline_visibility = Visibility::Hidden;
    }

    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);
    let dir = world_pos - player_pos;
    let angle = dir.y.atan2(dir.x);
    knife_transform.rotation = Quat::from_rotation_z(angle);
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

    let collision_dist = COLLISION_RADIUS * 1.8;

    for (entity, mut transform, hostile) in creature_queries.p1().iter_mut() {
        let creature_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let distance = player_pos.distance(creature_pos);

        if distance < HOSTILE_SIGHT_RANGE && distance > COLLISION_RADIUS {
            let dir = (player_pos - creature_pos).normalize();
            let movement = dir * hostile.speed * time.delta_secs();
            let new_pos = creature_pos + movement;

            let blocked = creature_positions.iter().any(|(other_entity, other_pos)| {
                *other_entity != entity && new_pos.distance(*other_pos) < collision_dist
            });

            if !blocked {
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
    mut player_query: Query<(Entity, &Transform, &mut Health), (With<Player>, Without<Creature>, Without<Dead>)>,
    hostile_query: Query<(&Transform, &Children), (With<Hostile>, Without<Dead>, Without<Stunned>)>,
    fist_query: Query<Entity, (With<Fist>, Without<WeaponSwing>)>,
    knockback_query: Query<&Knockback>,
) {
    let Ok((player_entity, player_transform, mut player_health)) = player_query.single_mut() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    if knockback_query.get(player_entity).is_ok() {
        return;
    }

    for (creature_transform, children) in &hostile_query {
        let creature_pos = Vec2::new(creature_transform.translation.x, creature_transform.translation.y);
        let distance = player_pos.distance(creature_pos);

        if distance < FIST_RANGE {
            for child in children.iter() {
                if let Ok(fist_entity) = fist_query.get(child) {
                    commands.entity(fist_entity).insert(WeaponSwing { timer: 0.0, duration: FIST_SWING_DURATION, base_angle: None });

                    player_health.0 -= 1;

                    if player_health.0 <= 0 {
                        commands.entity(player_entity).insert(Dead);
                    }

                    let knockback_dir = (player_pos - creature_pos).normalize();
                    commands.entity(player_entity).insert(Knockback {
                        velocity: knockback_dir * KNOCKBACK_FORCE,
                        timer: 0.0,
                    });
                    return;
                }
            }
        }
    }
}
