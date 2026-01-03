use bevy::prelude::*;

use crate::components::{Creature, Fist, HitCollider, Player, PlayerWeapon, StaticCollider, WalkCollider, Weapon};
use crate::resources::DebugConfig;

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

/// Toggle collision debug visibility with F3
pub fn toggle_collision_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_config: ResMut<DebugConfig>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        debug_config.show_collisions = !debug_config.show_collisions;
    }
}

/// Spawn debug circles for entities with collision
pub fn spawn_debug_circles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    debug_config: Res<DebugConfig>,
    // Player
    player_query: Query<(Entity, Option<&WalkCollider>, Option<&HitCollider>), (With<Player>, Without<HasDebugCircle>)>,
    // Creatures
    creature_query: Query<(Entity, Option<&WalkCollider>, Option<&HitCollider>), (With<Creature>, Without<HasDebugCircle>)>,
    // Static colliders (props)
    collider_query: Query<(Entity, &StaticCollider), Without<HasDebugCircle>>,
) {
    if !debug_config.show_collisions {
        return;
    }

    // Walk collision colors (yellow for all)
    let walk_color = materials.add(Color::srgba(1.0, 1.0, 0.0, 0.3));

    // Hit collision colors (brighter outline-like)
    let player_hit_color = materials.add(Color::srgba(0.0, 1.0, 0.5, 0.4));
    let creature_hit_color = materials.add(Color::srgba(1.0, 0.5, 0.0, 0.4));

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
            // Hit collision (outer, cyan ellipse)
            if let Some(hit) = hit_collider {
                let hit_mesh = meshes.add(Ellipse::new(hit.radius_x, hit.radius_y));
                parent.spawn((
                    HitDebugCircle,
                    Mesh2d(hit_mesh),
                    MeshMaterial2d(player_hit_color.clone()),
                    Transform::from_xyz(0.0, hit.offset_y, 9.9),
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
            // Hit collision (outer, orange ellipse)
            if let Some(hit) = hit_collider {
                let hit_mesh = meshes.add(Ellipse::new(hit.radius_x, hit.radius_y));
                parent.spawn((
                    HitDebugCircle,
                    Mesh2d(hit_mesh),
                    MeshMaterial2d(creature_hit_color.clone()),
                    Transform::from_xyz(0.0, hit.offset_y, 9.9),
                ));
            }
        });
    }

    // Static collider debug ellipses (props - only walk collision)
    for (entity, collider) in &collider_query {
        let ellipse_mesh = meshes.add(Ellipse::new(collider.radius_x, collider.radius_y));
        commands.entity(entity).insert(HasDebugCircle).with_children(|parent| {
            parent.spawn((
                CollisionDebugCircle,
                Mesh2d(ellipse_mesh),
                MeshMaterial2d(walk_color.clone()),
                Transform::from_xyz(0.0, collider.offset_y, 10.0),
            ));
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
    // Existing cones to despawn if weapon changed
    cone_query: Query<Entity, With<WeaponReachCone>>,
    // Creature fists
    fist_query: Query<(Entity, &Weapon), (With<Fist>, Without<HasDebugCone>)>,
) {
    if !debug_config.show_collisions {
        return;
    }

    let cone_color = materials.add(Color::srgba(1.0, 0.2, 0.2, 0.25));

    // Player weapon cone - spawn on player entity (combat uses player pos as origin)
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

                // CircularSector::new takes full angle, not half
                let cone_mesh = meshes.add(CircularSector::new(range, cone_angle));
                commands.entity(player_entity)
                    .insert(WeaponConeStats { range, half_angle: cone_angle })
                    .with_children(|parent| {
                        parent.spawn((
                            WeaponReachCone,
                            Mesh2d(cone_mesh),
                            MeshMaterial2d(cone_color.clone()),
                            Transform::from_xyz(0.0, 0.0, 9.8),
                        ));
                    });
            }
        }
    }

    // Creature fist circles (no cone check in combat, full circle range)
    for (entity, weapon) in &fist_query {
        let circle_mesh = meshes.add(Circle::new(weapon.range()));
        commands.entity(entity).insert(HasDebugCone).with_children(|parent| {
            parent.spawn((
                WeaponReachCone,
                Mesh2d(circle_mesh),
                MeshMaterial2d(cone_color.clone()),
                Transform::from_xyz(0.0, 0.0, 9.8),
            ));
        });
    }
}

/// Sync player's debug cone rotation with weapon rotation
pub fn update_player_weapon_cone(
    weapon_query: Query<&Transform, With<PlayerWeapon>>,
    player_query: Query<(&Transform, &Children), With<Player>>,
    mut cone_query: Query<&mut Transform, (With<WeaponReachCone>, Without<PlayerWeapon>, Without<Player>)>,
) {
    let Ok(weapon_transform) = weapon_query.single() else { return };
    let Ok((player_transform, children)) = player_query.single() else { return };

    for child in children.iter() {
        if let Ok(mut cone_transform) = cone_query.get_mut(child) {
            // CircularSector points +Y by default, weapon points +X, so offset by -90°
            let offset = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);
            cone_transform.rotation = weapon_transform.rotation * offset;
            // Counter any parent scale to keep cone at correct size
            cone_transform.scale = Vec3::new(
                1.0 / player_transform.scale.x,
                1.0 / player_transform.scale.y,
                1.0,
            );
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
