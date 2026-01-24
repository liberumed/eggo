use bevy::prelude::*;

use crate::combat::hit_detection::snap_to_cardinal;
use crate::inventory::weapons::{Fist, PlayerWeapon, Weapon, WeaponSwing};
use crate::core::{Dead, GameConfig, HitCollider, StaticCollider, WalkCollider};
use crate::creatures::{AttackOffset, CardinalAttacks, ContextMap, ContextMapCache, Creature, Hostile, NUM_DIRECTIONS};
use crate::player::{Player, PlayerSmashAttack, PlayerState};
use crate::props::{Prop, PropRegistry};
use crate::state_machine::StateMachine;
use super::config::DebugConfig;

/// Marker for debug collision circle (walk collision)
#[derive(Component)]
pub struct CollisionDebugCircle;

/// Marker for debug hit collision circle (hurtbox)
#[derive(Component)]
pub struct HitDebugCircle;

/// Marker for debug weapon reach cone
#[derive(Component)]
pub struct WeaponReachCone;

/// Marker for entities that have debug circles spawned
#[derive(Component)]
pub struct HasDebugCircle;

/// Marker for weapons that have debug cone spawned
#[derive(Component)]
pub struct HasDebugCone;

/// Stores current weapon stats for the debug cone (to detect changes)
#[derive(Component)]
pub struct WeaponConeStats {
    pub range: f32,
    pub half_angle: f32,
}

/// Links a debug circle to its owner creature
#[derive(Component)]
pub struct CreatureDebugCircle {
    pub entity: Entity,
    pub offset_y: f32,
    pub cardinal_attacks: bool,
}

/// Toggle collision debug visibility with F3
pub fn toggle_collision_debug(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_config: ResMut<DebugConfig>,
    debug_markers: Query<Entity, With<HasDebugCircle>>,
    debug_circles: Query<Entity, Or<(With<CollisionDebugCircle>, With<HitDebugCircle>)>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        debug_config.show_collisions = !debug_config.show_collisions;

        // When toggling off, clear markers so circles are recreated fresh when toggled on
        if !debug_config.show_collisions {
            for entity in &debug_markers {
                commands.entity(entity).remove::<HasDebugCircle>();
            }
            for entity in &debug_circles {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Spawn debug circles for entities with collision
pub fn spawn_debug_circles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    debug_config: Res<DebugConfig>,
    prop_registry: Res<PropRegistry>,
    // Player
    player_query: Query<(Entity, Option<&WalkCollider>, Option<&HitCollider>), (With<Player>, Without<HasDebugCircle>)>,
    // Creatures
    creature_query: Query<(Entity, Option<&WalkCollider>, Option<&HitCollider>), (With<Creature>, Without<HasDebugCircle>)>,
    // Static colliders (props)
    collider_query: Query<(Entity, &StaticCollider, Option<&Prop>), Without<HasDebugCircle>>,
) {
    if !debug_config.show_collisions {
        return;
    }

    // Walk collision colors (yellow for all)
    let walk_color = materials.add(Color::srgba(1.0, 1.0, 0.0, 0.3));

    // Hit collision colors (same green for both)
    let hit_color = materials.add(Color::srgba(0.0, 1.0, 0.5, 0.4));

    // Player debug ellipses
    for (entity, walk_collider, hit_collider) in &player_query {
        let (radius_x, radius_y, offset_y) = walk_collider
            .map(|w| (w.radius_x, w.radius_y, w.offset_y))
            .unwrap_or((8.0, 5.0, -11.0));
        let walk_mesh = meshes.add(Ellipse::new(radius_x, radius_y));
        commands.entity(entity).insert(HasDebugCircle).with_children(|parent| {
            // Walk collision (inner, green ellipse at feet)
            parent.spawn((
                CollisionDebugCircle,
                Mesh2d(walk_mesh),
                MeshMaterial2d(walk_color.clone()),
                Transform::from_xyz(0.0, offset_y, 10.0),
            ));
            // Hit collision (compound circles)
            if let Some(hit) = hit_collider {
                for circle in &hit.circles {
                    let hit_mesh = meshes.add(Circle::new(circle.radius));
                    parent.spawn((
                        HitDebugCircle,
                        Mesh2d(hit_mesh),
                        MeshMaterial2d(hit_color.clone()),
                        Transform::from_xyz(circle.offset.x, circle.offset.y, 10.1),
                    ));
                }
            }
        });
    }

    // Creature debug circles
    for (entity, walk_collider, hit_collider) in &creature_query {
        let (radius_x, radius_y, offset_y) = walk_collider
            .map(|w| (w.radius_x, w.radius_y, w.offset_y))
            .unwrap_or((8.0, 5.0, -11.0));
        let walk_mesh = meshes.add(Ellipse::new(radius_x, radius_y));
        commands.entity(entity).insert(HasDebugCircle).with_children(|parent| {
            // Walk collision (inner, red ellipse at feet)
            parent.spawn((
                CollisionDebugCircle,
                Mesh2d(walk_mesh),
                MeshMaterial2d(walk_color.clone()),
                Transform::from_xyz(0.0, offset_y, 10.0),
            ));
            // Hit collision (compound circles)
            if let Some(hit) = hit_collider {
                for circle in &hit.circles {
                    let hit_mesh = meshes.add(Circle::new(circle.radius));
                    parent.spawn((
                        HitDebugCircle,
                        Mesh2d(hit_mesh),
                        MeshMaterial2d(hit_color.clone()),
                        Transform::from_xyz(circle.offset.x, circle.offset.y, 10.1),
                    ));
                }
            }
        });
    }

    // Static collider debug ellipses (props - walk collision + hit radius)
    for (entity, collider, prop) in &collider_query {
        let ellipse_mesh = meshes.add(Ellipse::new(collider.radius_x, collider.radius_y));
        commands.entity(entity).insert(HasDebugCircle).with_children(|parent| {
            parent.spawn((
                CollisionDebugCircle,
                Mesh2d(ellipse_mesh),
                MeshMaterial2d(walk_color.clone()),
                Transform::from_xyz(collider.offset_x, collider.offset_y, 10.0),
            ));
            // Show hit radius from prop registry if defined
            if let Some(prop) = prop {
                if let Some(definition) = prop_registry.get(prop.prop_type) {
                    if let Some(hit_radius) = definition.hit_radius {
                        let hit_mesh = meshes.add(Circle::new(hit_radius));
                        parent.spawn((
                            HitDebugCircle,
                            Mesh2d(hit_mesh),
                            MeshMaterial2d(hit_color.clone()),
                            Transform::from_xyz(collider.offset_x, 0.0, 10.1),
                        ));
                    }
                }
            }
        });
    }
}

pub fn spawn_weapon_debug_cones(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<GameConfig>,
    debug_config: Res<DebugConfig>,
    player_query: Query<(Entity, Option<&WeaponConeStats>), With<Player>>,
    player_weapon_query: Query<&Weapon, With<PlayerWeapon>>,
    cone_query: Query<Entity, (With<WeaponReachCone>, Without<CreatureDebugCircle>)>,
    creature_query: Query<(Entity, &Children, Option<&CardinalAttacks>, Option<&AttackOffset>), (With<Creature>, Without<HasDebugCone>)>,
    fist_query: Query<&Weapon, With<Fist>>,
) {
    if !debug_config.show_collisions {
        return;
    }

    let cone_color = materials.add(Color::srgba(1.0, 0.2, 0.2, 0.25));

    // Player weapon cone - spawn on player entity, positioned at weapon origin
    if let Ok((player_entity, existing_stats)) = player_query.single() {
        if let Ok(weapon) = player_weapon_query.single() {
            let cone_angle = std::f32::consts::PI;  // Half-circle for player attacks
            let range = weapon.range();

            // Check if we need to create/recreate the cone
            let needs_spawn = match existing_stats {
                None => true,
                Some(stats) => (stats.range - range).abs() > 0.1 || (stats.half_angle - cone_angle).abs() > 0.01,
            };

            if needs_spawn {
                // Despawn existing player cone if any
                for cone_entity in &cone_query {
                    commands.entity(cone_entity).despawn();
                }

                let cone_mesh = meshes.add(CircularSector::new(range, cone_angle / 2.0));
                commands.entity(player_entity)
                    .insert(WeaponConeStats { range, half_angle: cone_angle })
                    .with_children(|parent| {
                        parent.spawn((
                            WeaponReachCone,
                            Mesh2d(cone_mesh),
                            MeshMaterial2d(cone_color.clone()),
                            Transform::from_xyz(0.0, config.attack_center_offset_y, 9.8),
                        ));
                    });
            }
        }
    }

    // Creature fist cones - spawn as independent entities (not children) to avoid scale issues
    for (creature_entity, children, cardinal_attacks, attack_offset) in &creature_query {
        // Find fist weapon in children
        for child in children.iter() {
            if let Ok(weapon) = fist_query.get(child) {
                let has_cardinal = cardinal_attacks.is_some();
                // Cardinal attackers use half-circle like player, other creatures use weapon cone
                let cone_angle = if has_cardinal {
                    std::f32::consts::PI
                } else {
                    weapon.cone_angle()
                };
                let offset_y = attack_offset.map(|o| o.0).unwrap_or(0.0);
                let cone_mesh = meshes.add(CircularSector::new(weapon.range(), cone_angle / 2.0));
                commands.entity(creature_entity).insert(HasDebugCone);
                // Spawn as independent entity, will follow creature position and rotation
                commands.spawn((
                    WeaponReachCone,
                    CreatureDebugCircle { entity: creature_entity, offset_y, cardinal_attacks: has_cardinal },
                    Mesh2d(cone_mesh),
                    MeshMaterial2d(cone_color.clone()),
                    Transform::from_xyz(0.0, 0.0, 9.8),
                ));
                break;
            }
        }
    }
}

/// Sync player debug cone rotation with aim direction (locked during swing)
pub fn update_player_debug_cone(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    weapon_query: Query<Option<&WeaponSwing>, With<PlayerWeapon>>,
    player_query: Query<(&Transform, &Children, &StateMachine<PlayerState>, Option<&PlayerSmashAttack>), (With<Player>, Without<PlayerWeapon>)>,
    mut cone_query: Query<&mut Transform, (With<WeaponReachCone>, Without<CreatureDebugCircle>, Without<Player>, Without<PlayerWeapon>)>,
) {
    let Ok(swing) = weapon_query.single() else { return };
    let Ok((player_transform, children, _state, smash_attack)) = player_query.single() else { return };

    // CircularSector points +Y by default, weapon points +X, so offset by -90°
    let cone_offset = -std::f32::consts::FRAC_PI_2;

    for child in children.iter() {
        if let Ok(mut cone_transform) = cone_query.get_mut(child) {
            // Check for sprite-based attack (Smash weapons)
            let aim_angle = if let Some(smash) = smash_attack {
                // Smash attack: use stored attack angle (already snapped to cardinal)
                smash.attack_angle
            } else if let Some(swing) = swing {
                // Slash/Stab weapon swing: use base_angle (already snapped to cardinal)
                if let Some(base_angle) = swing.base_angle {
                    base_angle
                } else {
                    continue; // No base angle during swing, skip
                }
            } else {
                // Compute aim angle from mouse position, snapped to cardinal
                let Ok(window) = windows.single() else { continue };
                let Ok((camera, camera_transform)) = camera_query.single() else { continue };
                let Some(cursor_pos) = window.cursor_position() else { continue };
                let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { continue };

                let player_pos = player_transform.translation.truncate();
                let dir = world_pos - player_pos;
                let raw_angle = dir.y.atan2(dir.x);
                snap_to_cardinal(raw_angle)
            };

            cone_transform.rotation = Quat::from_rotation_z(aim_angle + cone_offset);
            // Counter any parent scale to keep cone at correct size
            cone_transform.scale = Vec3::new(
                1.0 / player_transform.scale.x,
                1.0 / player_transform.scale.y,
                1.0,
            );
        }
    }
}

pub fn update_creature_debug_circles(
    mut commands: Commands,
    player_query: Query<&Transform, (With<Player>, Without<Creature>, Without<CreatureDebugCircle>)>,
    creature_query: Query<&Transform, (With<Creature>, Without<Dead>, Without<Player>, Without<CreatureDebugCircle>)>,
    mut circle_query: Query<(Entity, &CreatureDebugCircle, &mut Transform), (Without<Creature>, Without<Player>)>,
) {
    let player_pos = player_query.single().map(|t| t.translation.truncate()).unwrap_or_default();

    for (circle_entity, link, mut circle_transform) in &mut circle_query {
        if let Ok(creature_transform) = creature_query.get(link.entity) {
            let creature_pos = creature_transform.translation.truncate();

            circle_transform.translation.x = creature_pos.x;
            circle_transform.translation.y = creature_pos.y + link.offset_y;

            // Rotate cone toward player (same direction as attack)
            // CircularSector points +Y by default, so offset by -90° to align with attack direction
            let dir = player_pos - creature_pos;
            let raw_angle = dir.y.atan2(dir.x);
            // Cardinal attackers snap to cardinal directions like player
            let angle = if link.cardinal_attacks {
                snap_to_cardinal(raw_angle)
            } else {
                raw_angle
            };
            let offset = -std::f32::consts::FRAC_PI_2;
            circle_transform.rotation = Quat::from_rotation_z(angle + offset);
        } else {
            // Creature is dead or despawned, remove debug circle
            commands.entity(circle_entity).despawn();
        }
    }
}

/// Update visibility of debug circles
pub fn update_debug_visibility(
    debug_config: Res<DebugConfig>,
    mut walk_query: Query<&mut Visibility, (With<CollisionDebugCircle>, Without<HitDebugCircle>, Without<WeaponReachCone>)>,
    mut hit_query: Query<&mut Visibility, (With<HitDebugCircle>, Without<CollisionDebugCircle>, Without<WeaponReachCone>)>,
    mut cone_query: Query<&mut Visibility, (With<WeaponReachCone>, Without<CollisionDebugCircle>, Without<HitDebugCircle>)>,
) {
    let visibility = if debug_config.show_collisions {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    for mut vis in &mut walk_query {
        *vis = visibility;
    }
    for mut vis in &mut hit_query {
        *vis = visibility;
    }
    for mut vis in &mut cone_query {
        *vis = visibility;
    }
}

// ============================================================================
// Context Steering Debug Visualization
// ============================================================================

/// Marker for steering debug ray entities
#[derive(Component)]
pub struct SteeringDebugRay {
    pub direction_index: usize,
}

/// Marker for creatures that have steering debug spawned
#[derive(Component)]
pub struct HasSteeringDebug;

/// Links steering debug rays to their owner creature
#[derive(Component)]
pub struct SteeringDebugOwner(pub Entity);

const STEERING_RAY_LENGTH: f32 = 25.0;
const STEERING_RAY_WIDTH: f32 = 2.0;

/// Spawn steering debug rays for hostile creatures with context map cache
pub fn spawn_steering_debug(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    debug_config: Res<DebugConfig>,
    hostile_query: Query<Entity, (With<Hostile>, With<ContextMapCache>, Without<HasSteeringDebug>)>,
) {
    if !debug_config.show_collisions {
        return;
    }

    // Create ray mesh (rectangle pointing in +X direction)
    let ray_mesh = meshes.add(Rectangle::new(STEERING_RAY_LENGTH, STEERING_RAY_WIDTH));
    let neutral_color = materials.add(Color::srgba(0.5, 0.5, 0.5, 0.5));

    for entity in &hostile_query {
        commands.entity(entity).insert(HasSteeringDebug);

        // Spawn 8 rays as independent entities
        for i in 0..NUM_DIRECTIONS {
            commands.spawn((
                SteeringDebugRay { direction_index: i },
                SteeringDebugOwner(entity),
                Mesh2d(ray_mesh.clone()),
                MeshMaterial2d(neutral_color.clone()),
                Transform::from_xyz(0.0, 0.0, 9.5),
            ));
        }
    }
}

/// Update steering debug ray positions, rotations, and colors
pub fn update_steering_debug(
    mut commands: Commands,
    debug_config: Res<DebugConfig>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    hostile_query: Query<(&Transform, &ContextMapCache), (With<Hostile>, Without<Dead>, Without<SteeringDebugRay>)>,
    mut ray_query: Query<(Entity, &SteeringDebugOwner, &SteeringDebugRay, &mut Transform, &mut MeshMaterial2d<ColorMaterial>), Without<Hostile>>,
) {
    if !debug_config.show_collisions {
        return;
    }

    for (ray_entity, owner, ray, mut ray_transform, mut material) in &mut ray_query {
        if let Ok((creature_transform, context_cache)) = hostile_query.get(owner.0) {
            let creature_pos = creature_transform.translation.truncate();
            let context = &context_cache.0;

            // Get direction for this ray
            let dir = ContextMap::direction(ray.direction_index);
            let angle = dir.y.atan2(dir.x);

            // Position ray at creature center, offset by half ray length in direction
            ray_transform.translation.x = creature_pos.x + dir.x * STEERING_RAY_LENGTH * 0.5;
            ray_transform.translation.y = creature_pos.y + dir.y * STEERING_RAY_LENGTH * 0.5;
            ray_transform.rotation = Quat::from_rotation_z(angle);

            // Color based on interest vs danger
            let interest = context.interest[ray.direction_index];
            let danger = context.danger[ray.direction_index];
            let value = interest - danger;

            let color = if value > 0.0 {
                // Green for positive (interest > danger)
                Color::srgba(0.0, value.min(1.0), 0.0, 0.6)
            } else if danger > 0.0 {
                // Red for danger
                Color::srgba(danger.min(1.0), 0.0, 0.0, 0.6)
            } else {
                // Gray for neutral
                Color::srgba(0.3, 0.3, 0.3, 0.3)
            };

            *material = MeshMaterial2d(materials.add(color));
        } else {
            // Owner no longer exists or is dead, despawn ray
            commands.entity(ray_entity).despawn();
        }
    }
}

/// Clean up steering debug when toggling off
pub fn cleanup_steering_debug(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    debug_config: Res<DebugConfig>,
    ray_query: Query<Entity, With<SteeringDebugRay>>,
    marker_query: Query<Entity, With<HasSteeringDebug>>,
) {
    if keyboard.just_pressed(KeyCode::F3) && debug_config.show_collisions {
        // Debug was just turned off (show_collisions is the NEW state after toggle)
        // Actually, toggle happens before this runs, so check if it's now OFF
    }

    // When debug is off, clean up
    if !debug_config.show_collisions {
        for entity in &ray_query {
            commands.entity(entity).despawn();
        }
        for entity in &marker_query {
            commands.entity(entity).remove::<HasSteeringDebug>();
        }
    }
}
