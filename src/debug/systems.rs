use bevy::prelude::*;

use crate::combat::{Fist, PlayerWeapon, Weapon, WeaponSwing};
use crate::constants::WEAPON_OFFSET;
use crate::core::{Dead, HitCollider, StaticCollider, WalkCollider};
use crate::creatures::Creature;
use crate::player::{Player, PlayerAttackState};
use crate::props::{Prop, PropRegistry};
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
pub struct CreatureDebugCircle(pub Entity);

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
            // Hit collision (outer ellipse) - higher Z to be visible above walk
            if let Some(hit) = hit_collider {
                let hit_mesh = meshes.add(Ellipse::new(hit.radius_x, hit.radius_y));
                parent.spawn((
                    HitDebugCircle,
                    Mesh2d(hit_mesh),
                    MeshMaterial2d(hit_color.clone()),
                    Transform::from_xyz(0.0, hit.offset_y, 10.1),
                ));
            }
        });
    }

    // Creature debug ellipses
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
            // Hit collision (outer ellipse) - slightly larger for visibility
            if let Some(hit) = hit_collider {
                let hit_mesh = meshes.add(Ellipse::new(hit.radius_x + 2.0, hit.radius_y + 2.0));
                parent.spawn((
                    HitDebugCircle,
                    Mesh2d(hit_mesh),
                    MeshMaterial2d(hit_color.clone()),
                    Transform::from_xyz(0.0, hit.offset_y, 10.1),
                ));
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

/// Spawn debug cones for weapons
pub fn spawn_weapon_debug_cones(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    debug_config: Res<DebugConfig>,
    // Player (to spawn cone on player, not weapon)
    player_query: Query<(Entity, Option<&WeaponConeStats>), With<Player>>,
    player_weapon_query: Query<&Weapon, With<PlayerWeapon>>,
    // Existing player cones to despawn if weapon changed (exclude creature circles)
    cone_query: Query<Entity, (With<WeaponReachCone>, Without<CreatureDebugCircle>)>,
    // Creatures with fists (spawn circle on creature, not fist)
    creature_query: Query<(Entity, &Children), (With<Creature>, Without<HasDebugCone>)>,
    fist_query: Query<&Weapon, With<Fist>>,
) {
    if !debug_config.show_collisions {
        return;
    }

    let cone_color = materials.add(Color::srgba(1.0, 0.2, 0.2, 0.25));

    // Player weapon cone - spawn on player entity, positioned at weapon origin
    if let Ok((player_entity, existing_stats)) = player_query.single() {
        if let Ok(weapon) = player_weapon_query.single() {
            let cone_angle = weapon.cone_angle();  // Full angle, not half
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

                // CircularSector::new takes half angle, so divide by 2
                let cone_mesh = meshes.add(CircularSector::new(range, cone_angle / 2.0));
                // Use fixed weapon offset (not affected by block pull-back)
                let weapon_offset = Vec2::new(WEAPON_OFFSET.0, WEAPON_OFFSET.1);
                commands.entity(player_entity)
                    .insert(WeaponConeStats { range, half_angle: cone_angle })
                    .with_children(|parent| {
                        parent.spawn((
                            WeaponReachCone,
                            Mesh2d(cone_mesh),
                            MeshMaterial2d(cone_color.clone()),
                            Transform::from_xyz(weapon_offset.x, weapon_offset.y, 9.8),
                        ));
                    });
            }
        }
    }

    // Creature fist cones - spawn as independent entities (not children) to avoid scale issues
    for (creature_entity, children) in &creature_query {
        // Find fist weapon in children
        for child in children.iter() {
            if let Ok(weapon) = fist_query.get(child) {
                // Use cone like player, not circle
                let cone_mesh = meshes.add(CircularSector::new(weapon.range(), weapon.cone_angle() / 2.0));
                commands.entity(creature_entity).insert(HasDebugCone);
                // Spawn as independent entity, will follow creature position and rotation
                commands.spawn((
                    WeaponReachCone,
                    CreatureDebugCircle(creature_entity),
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
    player_query: Query<(&Transform, &Children, Option<&PlayerAttackState>), (With<Player>, Without<PlayerWeapon>)>,
    mut cone_query: Query<&mut Transform, (With<WeaponReachCone>, Without<CreatureDebugCircle>, Without<Player>, Without<PlayerWeapon>)>,
) {
    let Ok(swing) = weapon_query.single() else { return };
    let Ok((player_transform, children, attack_state)) = player_query.single() else { return };

    // CircularSector points +Y by default, weapon points +X, so offset by -90°
    let cone_offset = -std::f32::consts::FRAC_PI_2;

    for child in children.iter() {
        if let Ok(mut cone_transform) = cone_query.get_mut(child) {
            // Check for sprite-based attack (Smash weapons)
            let aim_angle = if let Some(attack) = attack_state {
                // Smash attack: locked to left or right
                if attack.facing_right { 0.0 } else { std::f32::consts::PI }
            } else if let Some(swing) = swing {
                // Slash/Stab weapon swing: use base_angle
                if let Some(base_angle) = swing.base_angle {
                    base_angle
                } else {
                    continue; // No base angle during swing, skip
                }
            } else {
                // Compute aim angle directly from mouse position (same as arc indicator)
                let Ok(window) = windows.single() else { continue };
                let Ok((camera, camera_transform)) = camera_query.single() else { continue };
                let Some(cursor_pos) = window.cursor_position() else { continue };
                let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { continue };

                let player_pos = player_transform.translation.truncate();
                let dir = world_pos - player_pos;
                dir.y.atan2(dir.x)
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

/// Sync creature debug cones with creature positions/rotations and despawn when creature dies
pub fn update_creature_debug_circles(
    mut commands: Commands,
    player_query: Query<&Transform, (With<Player>, Without<Creature>, Without<CreatureDebugCircle>)>,
    creature_query: Query<&Transform, (With<Creature>, Without<Dead>, Without<Player>, Without<CreatureDebugCircle>)>,
    mut circle_query: Query<(Entity, &CreatureDebugCircle, &mut Transform), (Without<Creature>, Without<Player>)>,
) {
    let player_pos = player_query.single().map(|t| t.translation.truncate()).unwrap_or_default();

    for (circle_entity, link, mut circle_transform) in &mut circle_query {
        if let Ok(creature_transform) = creature_query.get(link.0) {
            let creature_pos = creature_transform.translation.truncate();
            circle_transform.translation.x = creature_pos.x;
            circle_transform.translation.y = creature_pos.y;
            // Rotate cone toward player (same direction as attack)
            // CircularSector points +Y by default, so offset by -90° to align with attack direction
            let dir = player_pos - creature_pos;
            let angle = dir.y.atan2(dir.x);
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
