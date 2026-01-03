use bevy::prelude::*;

use crate::components::{Creature, HitCollider, Player, StaticCollider, WalkCollider};
use crate::resources::DebugConfig;

/// Marker for debug collision circle (walk collision)
#[derive(Component)]
pub struct CollisionDebugCircle;

/// Marker for debug hit collision circle (hurtbox)
#[derive(Component)]
pub struct HitDebugCircle;

/// Marker for entities that have debug circles spawned
#[derive(Component)]
pub struct HasDebugCircle;

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

/// Update visibility of debug circles
pub fn update_debug_visibility(
    debug_config: Res<DebugConfig>,
    mut walk_query: Query<&mut Visibility, (With<CollisionDebugCircle>, Without<HitDebugCircle>)>,
    mut hit_query: Query<&mut Visibility, (With<HitDebugCircle>, Without<CollisionDebugCircle>)>,
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
}
