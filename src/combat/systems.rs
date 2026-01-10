use bevy::prelude::*;
use rand::Rng;

use crate::constants::*;
use crate::core::{ellipse_push, Blocking, Dead, DeathAnimation, GameAction, Health, HitCollider, InputBindings, Knockback, StaticCollider, Stunned};
use crate::creatures::{ContextMapCache, Creature, FlankPreference, Hostile};
use crate::player::{Player, PlayerSmashAttack, PlayerState};
use crate::state_machine::StateMachine;
use crate::props::{BarrelSprite, CrateSprite, Crate2Sprite, Destructible, Prop, PropRegistry, PropType};
use super::{CreatureRangeIndicator, PlayerRangeIndicator, WeaponRangeIndicator};
use crate::inventory::weapons::{weapon_catalog, AttackType, Drawn, Fist, PlayerWeapon, Weapon, WeaponSwing, WeaponVisualMesh};
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

pub fn apply_mesh_attack_hits(
    mut commands: Commands,
    mut hitstop: ResMut<Hitstop>,
    mut screen_shake: ResMut<ScreenShake>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<(Entity, &Transform, &StateMachine<PlayerState>), (With<Player>, Without<Dead>, Without<DeathAnimation>)>,
    mut weapon_query: Query<(&Transform, &Weapon, &mut WeaponSwing), With<PlayerWeapon>>,
    mut creatures_query: Query<(Entity, &Transform, &mut Health, Option<&Hostile>, Option<&HitCollider>, Option<&crate::creatures::ProvokedSteering>), (With<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    assets: Res<CharacterAssets>,
) {
    use crate::state_machine::AttackPhase;
    use crate::creatures::CreatureSteering;

    let Ok((player_entity, player_transform, state)) = player_query.single() else { return };

    if !matches!(state.current(), PlayerState::Attacking(AttackPhase::Strike)) {
        return;
    }

    let Ok((weapon_transform, weapon, mut swing)) = weapon_query.single_mut() else { return };

    if swing.hit_applied {
        return;
    }
    swing.hit_applied = true;

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

    for (entity, creature_transform, mut health, hostile, hit_collider, provoked_steering) in &mut creatures_query {
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
                // Make non-hostile creature become hostile when hit (provoked = direct pursuit)
                commands.entity(entity).insert((Hostile { speed: PROVOKED_SPEED }, crate::creatures::Provoked));

                // Swap steering config to provoked behavior
                if let Some(provoked_config) = provoked_steering {
                    commands.entity(entity).insert(CreatureSteering(provoked_config.0.clone()));
                }

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

pub fn apply_smash_attack_hits(
    mut commands: Commands,
    mut hitstop: ResMut<Hitstop>,
    mut screen_shake: ResMut<ScreenShake>,
    prop_registry: Res<PropRegistry>,
    mut player_query: Query<(Entity, &Transform, &mut PlayerSmashAttack, &StateMachine<PlayerState>), With<Player>>,
    weapon_query: Query<(Entity, &Weapon), With<PlayerWeapon>>,
    mut creatures_query: Query<(Entity, &Transform, &mut Health, Option<&Hostile>, Option<&HitCollider>, Option<&crate::creatures::ProvokedSteering>), (With<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    mut props_query: Query<(Entity, &Transform, &Prop, &mut Destructible, Option<&mut CrateSprite>, Option<&mut Crate2Sprite>, Option<&mut BarrelSprite>, Option<&mut Sprite>), Without<Creature>>,
    assets: Res<CharacterAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    use crate::state_machine::AttackPhase;
    use crate::creatures::CreatureSteering;
    let Ok((player_entity, player_transform, mut smash, state)) = player_query.single_mut() else { return };

    if !matches!(state.current(), PlayerState::Attacking(AttackPhase::Strike)) {
        return;
    }

    if smash.hit_applied {
        return;
    }
    smash.hit_applied = true;

    let Ok((_, weapon)) = weapon_query.single() else { return };

    let attack_dir = if smash.facing_right {
        Vec2::X
    } else {
        Vec2::NEG_X
    };

    let attack_origin = player_transform.translation.truncate();
    let hit_cone = HitCone::new(attack_origin, attack_dir, weapon.range(), weapon.cone_angle());

    let mut rng = rand::rng();
    let mut hit_any = false;

    for (entity, creature_transform, mut health, hostile, hit_collider, provoked_steering) in &mut creatures_query {
        let creature_pos = creature_transform.translation.truncate();
        let hit_radius = hit_collider.map(|h| h.radius_x.max(h.radius_y)).unwrap_or(0.0);

        if hit_cone.hits(creature_pos, hit_radius) {
            hit_any = true;
            health.0 -= weapon.damage;

            let knockback_dir = (creature_pos - hit_cone.origin).normalize_or_zero();
            weapon.apply_on_hit(&mut commands, entity, knockback_dir);

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
                commands.entity(entity).insert((Hostile { speed: PROVOKED_SPEED }, crate::creatures::Provoked));

                if let Some(provoked_config) = provoked_steering {
                    commands.entity(entity).insert(CreatureSteering(provoked_config.0.clone()));
                }

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

    for (entity, prop_transform, prop, mut destructible, crate_sprite, crate2_sprite, barrel_sprite, sprite) in &mut props_query {
        let prop_pos = prop_transform.translation.truncate();

        let Some(definition) = prop_registry.get(prop.prop_type) else { continue };
        let Some(hit_radius) = definition.hit_radius else { continue };

        if hit_cone.hits(prop_pos, hit_radius) {
            hit_any = true;
            destructible.health -= weapon.damage;

            if destructible.health <= 0 {
                commands.entity(entity).despawn();
            } else if prop.prop_type == PropType::Crate {
                if let (Some(mut crate_state), Some(mut sprite)) = (crate_sprite, sprite) {
                    if !crate_state.damaged {
                        crate_state.damaged = true;
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            atlas.index = 1;
                        }
                    }
                }
            } else if prop.prop_type == PropType::Crate2 {
                if let (Some(mut crate2_state), Some(mut sprite)) = (crate2_sprite, sprite) {
                    if !crate2_state.damaged {
                        crate2_state.damaged = true;
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            atlas.index = 1;
                        }
                    }
                }
            } else if prop.prop_type == PropType::Barrel {
                if let (Some(mut barrel_state), Some(mut sprite)) = (barrel_sprite, sprite) {
                    if !barrel_state.damaged {
                        barrel_state.damaged = true;
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            atlas.index = 1;
                        }
                    }
                }
            }
        }
    }

    if hit_any {
        let recoil_force = weapon.knockback_force() * 0.15;
        commands.entity(player_entity).insert(Knockback {
            velocity: -attack_dir * recoil_force,
            timer: 0.0,
        });

        hitstop.trigger(HITSTOP_DURATION);
        screen_shake.trigger(SCREEN_SHAKE_INTENSITY, SCREEN_SHAKE_DURATION);
    }
}

pub fn aim_weapon(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    player_query: Query<(Entity, &Transform, &StateMachine<PlayerState>), With<Player>>,
    mut weapon_query: Query<(&mut Transform, &Weapon), (With<PlayerWeapon>, Without<Player>, Without<WeaponSwing>, Without<TargetOutline>)>,
    mut outline_query: Query<&mut Visibility, With<TargetOutline>>,
    blocking_query: Query<&Blocking>,
) {
    let Ok((player_entity, player_transform, state)) = player_query.single() else { return };

    // Don't aim weapon during attack or dash
    if matches!(state.current(), PlayerState::Attacking(_) | PlayerState::Dashing) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
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
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Creature>, Without<StaticCollider>)>,
    collider_query: Query<(&Transform, &StaticCollider), (Without<Player>, Without<Creature>)>,
    mut creature_queries: ParamSet<(
        Query<(Entity, &Transform), (With<Creature>, Without<Dead>, Without<StaticCollider>)>,
        Query<(Entity, &mut Transform, &Hostile, &crate::creatures::CreatureSteering, &crate::state_machine::StateMachine<crate::creatures::CreatureState>, Option<&mut ContextMapCache>, Option<&FlankPreference>), (Without<Dead>, Without<Player>, Without<Stunned>, Without<StaticCollider>)>,
    )>,
) {
    use crate::creatures::{ContextMap, ContextMapCache, CreatureState, FlankPreference, SteeringStrategy, seek_interest, seek_with_flank, obstacle_danger, separation_danger, player_proximity_danger, occupied_angle_danger};
    use rand::Rng;

    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();

    // Gather all creature positions for separation behavior
    let creature_positions: Vec<(Entity, Vec2)> = creature_queries
        .p0()
        .iter()
        .map(|(e, t)| (e, t.translation.truncate()))
        .collect();

    // Gather static collider data for obstacle avoidance
    let collider_data: Vec<(Vec2, Vec2)> = collider_query
        .iter()
        .map(|(t, c)| (Vec2::new(t.translation.x, t.translation.y + c.offset_y), Vec2::new(c.radius_x, c.radius_y)))
        .collect();

    for (entity, mut transform, hostile, steering, state_machine, context_cache, flank_pref) in creature_queries.p1().iter_mut() {
        // Only process creatures in Chase state
        if *state_machine.current() != CreatureState::Chase {
            continue;
        }

        let config = &steering.0;
        let creature_pos = transform.translation.truncate();
        let distance = player_pos.distance(creature_pos);

        if distance > config.min_player_distance {
            // Build context map
            let mut context = ContextMap::new();

            // Interest: use strategy from config
            match config.strategy {
                SteeringStrategy::Direct => {
                    seek_interest(&mut context, creature_pos, player_pos);
                }
                SteeringStrategy::Flanking => {
                    let flank_angle = if let Some(pref) = flank_pref {
                        pref.0
                    } else {
                        // Assign flank angle from config range
                        let mut rng = rand::rng();
                        let sign = if rng.random_bool(0.5) { 1.0 } else { -1.0 };
                        let magnitude = rng.random_range(config.flank_angle_min..config.flank_angle_max);
                        let angle = sign * magnitude;
                        commands.entity(entity).insert(FlankPreference(angle));
                        angle
                    };
                    seek_with_flank(&mut context, creature_pos, player_pos, flank_angle);
                }
            }

            // Danger: avoid obstacles
            obstacle_danger(&mut context, creature_pos, &collider_data, config.obstacle_look_ahead);

            // Danger: separation from other creatures
            let others: Vec<Vec2> = creature_positions.iter()
                .filter(|(e, _)| *e != entity)
                .map(|(_, p)| *p)
                .collect();
            separation_danger(&mut context, creature_pos, &others, config.separation_radius);

            // Danger: avoid approaching from same angle as other creatures (forces spreading)
            occupied_angle_danger(&mut context, creature_pos, player_pos, &others, config.occupied_angle_spread);

            // Danger: don't get too close to player
            player_proximity_danger(&mut context, creature_pos, player_pos, config.min_player_distance);

            // Resolve and move
            let (direction, strength) = context.resolve();
            if strength > 0.0 {
                let movement = direction * hostile.speed * strength * time.delta_secs();
                let mut new_pos = creature_pos + movement;

                // Still apply push-based collision for safety (handles edge cases)
                let creature_radius = Vec2::new(8.0, 5.0);
                let creature_offset_y = -11.0;
                for (collider_pos, collider_radius) in &collider_data {
                    let creature_feet = Vec2::new(new_pos.x, new_pos.y + creature_offset_y);
                    let push = ellipse_push(creature_feet, creature_radius, *collider_pos, *collider_radius);
                    new_pos += push;
                }

                transform.translation.x = new_pos.x;
                transform.translation.y = new_pos.y;
            }

            // Cache context map for debug visualization
            if let Some(mut cache) = context_cache {
                cache.0 = context;
            } else {
                commands.entity(entity).insert(ContextMapCache(context));
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

/// Requests transition to Attack state when creature detects player in range.
/// Reads PlayerInRange events emitted by detect_player_proximity system.
pub fn hostile_attack(
    mut transitions: MessageWriter<crate::state_machine::RequestTransition<crate::creatures::CreatureState>>,
    mut player_in_range: MessageReader<crate::creatures::PlayerInRange>,
) {
    use crate::state_machine::{AttackPhase, RequestTransition};
    use crate::creatures::CreatureState;

    for event in player_in_range.read() {
        // Request transition to Attack state
        transitions.write(RequestTransition::new(
            event.creature,
            CreatureState::Attack(AttackPhase::WindUp),
        ));
    }
}

/// Check if player successfully blocks an attack from given direction
/// Returns (damage_mult, knockback_mult, was_blocked)
fn calculate_block(
    player_pos: Vec2,
    attacker_pos: Vec2,
    is_blocking: bool,
    player_weapon: Option<(&Weapon, &Transform)>,
) -> (f32, f32, bool) {
    if !is_blocking {
        return (1.0, 1.0, false);
    }

    let Some((weapon, transform)) = player_weapon else {
        return (1.0, 1.0, false);
    };

    // Get facing direction from weapon angle
    let (_, angle) = transform.rotation.to_axis_angle();
    let visual_angle = if transform.rotation.z < 0.0 { -angle } else { angle };
    let facing_dir = angle_to_direction(visual_angle - BLOCK_FACING_OFFSET);

    // Check if facing attacker
    let to_attacker = attacker_pos - player_pos;
    let to_attacker_len = to_attacker.length();

    if to_attacker_len > 0.001 && facing_dir.dot(to_attacker) > BLOCK_ANGLE_THRESHOLD * to_attacker_len {
        let dmg_mult = 1.0 - weapon.block_damage_reduction();
        let kb_mult = 1.0 - weapon.block_knockback_reduction();
        (dmg_mult, kb_mult, true)
    } else {
        (1.0, 1.0, false)
    }
}

/// Apply creature attack effects to player (damage, knockback, visual effects)
fn apply_attack_to_player(
    commands: &mut Commands,
    player_entity: Entity,
    attacker_entity: Entity,
    player_pos: Vec2,
    attacker_pos: Vec2,
    weapon: &Weapon,
    damage_mult: f32,
    knockback_mult: f32,
    blocked: bool,
    player_health: &mut Health,
    hitstop: &mut Hitstop,
    screen_shake: &mut ScreenShake,
) {
    // Apply damage
    let final_damage = ((weapon.damage as f32) * damage_mult).floor() as i32;
    player_health.0 -= final_damage.max(0);

    // Knockback player
    let knockback_dir = (player_pos - attacker_pos).normalize();
    commands.entity(player_entity).insert(Knockback {
        velocity: knockback_dir * weapon.knockback_force() * knockback_mult,
        timer: 0.0,
    });

    // Knockback attacker if blocked
    if blocked {
        commands.entity(attacker_entity).insert(Knockback {
            velocity: -knockback_dir * BLOCK_KNOCKBACK,
            timer: 0.0,
        });
    }

    // Effects
    hitstop.trigger(HITSTOP_DURATION);
    screen_shake.trigger(SCREEN_SHAKE_INTENSITY, SCREEN_SHAKE_DURATION);

    commands.entity(player_entity).insert(HitHighlight {
        timer: 0.0,
        duration: HIT_HIGHLIGHT_DURATION,
        original_material: None,
    });
}

/// Process creature attacks against player
/// Only one attack can hit per frame (prevents stun-lock from multiple creatures)
/// Only applies hits during Attack(Strike) phase
pub fn process_creature_attacks(
    mut commands: Commands,
    mut hitstop: ResMut<Hitstop>,
    mut screen_shake: ResMut<ScreenShake>,
    mut player_query: Query<(Entity, &Transform, &mut Health, Option<&HitCollider>, &StateMachine<PlayerState>), (With<Player>, Without<Creature>, Without<Dead>, Without<DeathAnimation>)>,
    hostile_query: Query<(Entity, &Transform, &crate::state_machine::StateMachine<crate::creatures::CreatureState>), (With<Hostile>, Without<Dead>, Without<Stunned>)>,
    mut fist_query: Query<(&Weapon, &mut WeaponSwing, &ChildOf), With<Fist>>,
    knockback_query: Query<&Knockback>,
    blocking_query: Query<&Blocking>,
    player_weapon_query: Query<(&Weapon, &Transform), With<PlayerWeapon>>,
) {
    use crate::state_machine::AttackPhase;
    use crate::creatures::CreatureState;

    let Ok((player_entity, player_transform, mut player_health, player_hit_collider, player_state)) = player_query.single_mut() else { return };
    let player_pos = player_transform.translation.truncate();
    let player_hit_radius = player_hit_collider.map(|h| h.radius_x.max(h.radius_y)).unwrap_or(0.0);

    // Player invincible during dash or knockback
    if *player_state.current() == PlayerState::Dashing || knockback_query.get(player_entity).is_ok() {
        return;
    }

    // Check blocking state once
    let is_blocking = blocking_query.get(player_entity).is_ok();
    let player_weapon = player_weapon_query.single().ok();

    // Iterate fists directly (instead of nested children loop)
    for (weapon, mut swing, child_of) in &mut fist_query {
        // Skip if hit already applied
        if swing.hit_applied {
            continue;
        }

        // Get attacker (parent creature) and check state
        let Ok((attacker_entity, attacker_transform, state_machine)) = hostile_query.get(child_of.parent()) else {
            continue;
        };

        // Only apply hit during Strike phase
        if *state_machine.current() != CreatureState::Attack(AttackPhase::Strike) {
            continue;
        }

        let attacker_pos = attacker_transform.translation.truncate();

        // Mark hit as applied
        swing.hit_applied = true;

        // Hit detection: cone attack toward player
        let attack_dir = (player_pos - attacker_pos).normalize_or_zero();
        let hit_cone = HitCone::new(attacker_pos, attack_dir, weapon.range(), weapon.cone_angle());

        if !hit_cone.hits(player_pos, player_hit_radius) {
            continue;
        }

        // Calculate block result
        let (damage_mult, knockback_mult, blocked) = calculate_block(
            player_pos,
            attacker_pos,
            is_blocking,
            player_weapon,
        );

        // Apply damage and effects
        apply_attack_to_player(
            &mut commands,
            player_entity,
            attacker_entity,
            player_pos,
            attacker_pos,
            weapon,
            damage_mult,
            knockback_mult,
            blocked,
            &mut player_health,
            &mut hitstop,
            &mut screen_shake,
        );

        // Only one hit per frame
        return;
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
