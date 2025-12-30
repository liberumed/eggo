use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::spawners::CharacterAssets;

pub fn knife_attack(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, (With<Player>, Without<Dead>)>,
    knife_query: Query<(Entity, &Transform), With<Knife>>,
    knife_swing_query: Query<&WeaponSwing, With<Knife>>,
    mut creatures_query: Query<(Entity, &Transform, &mut Health, Option<&Hostile>), (With<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    assets: Res<CharacterAssets>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    if knife_swing_query.iter().next().is_some() {
        return;
    }

    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    let knife_dir = if let Ok((knife_entity, knife_transform)) = knife_query.single() {
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

        if player_pos.distance(creature_pos) < KNIFE_RANGE {
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
                commands.entity(entity).insert(Hostile);

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
    player_query: Query<&Transform, With<Player>>,
    mut knife_query: Query<&mut Transform, (With<Knife>, Without<Player>, Without<WeaponSwing>, Without<TargetOutline>)>,
    creatures_query: Query<&Transform, (With<Creature>, Without<Dead>, Without<Player>, Without<Knife>, Without<TargetOutline>)>,
    mut outline_query: Query<(&mut Transform, &mut Visibility), (With<TargetOutline>, Without<Knife>, Without<Player>, Without<Creature>)>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let Ok(mut knife_transform) = knife_query.single_mut() else { return };
    let Ok((mut outline_transform, mut outline_visibility)) = outline_query.single_mut() else { return };

    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    let closest = creatures_query
        .iter()
        .min_by(|a, b| {
            let pos_a = Vec2::new(a.translation.x, a.translation.y);
            let pos_b = Vec2::new(b.translation.x, b.translation.y);
            pos_a.distance(player_pos)
                .partial_cmp(&pos_b.distance(player_pos))
                .unwrap()
        });

    if let Some(target) = closest {
        let target_pos = Vec2::new(target.translation.x, target.translation.y);
        let dir = target_pos - player_pos;
        let angle = dir.y.atan2(dir.x);
        knife_transform.rotation = Quat::from_rotation_z(angle);

        outline_transform.translation.x = target_pos.x;
        outline_transform.translation.y = target_pos.y;
        outline_transform.scale = target.scale;
        outline_transform.rotation = target.rotation;
        *outline_visibility = Visibility::Inherited;
    } else {
        *outline_visibility = Visibility::Hidden;
    }
}

pub fn hostile_ai(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Creature>)>,
    mut creature_queries: ParamSet<(
        Query<(Entity, &Transform), (With<Creature>, Without<Dead>)>,
        Query<(Entity, &mut Transform), (With<Hostile>, Without<Dead>, Without<Player>, Without<Stunned>)>,
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

    for (entity, mut transform) in creature_queries.p1().iter_mut() {
        let creature_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let distance = player_pos.distance(creature_pos);

        if distance < HOSTILE_SIGHT_RANGE && distance > COLLISION_RADIUS {
            let dir = (player_pos - creature_pos).normalize();
            let movement = dir * HOSTILE_SPEED * time.delta_secs();
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
