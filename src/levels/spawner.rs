use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;

use super::LevelData;

#[derive(Component)]
pub struct LevelBackground;

#[derive(Component)]
pub struct VoidBackground;

#[derive(Component)]
pub struct WinZone {
    pub radius: f32,
}

#[derive(Component)]
pub struct WinZoneTimerText;

#[derive(Resource, Default)]
pub struct WinZoneTimer(pub f32);

const Z_VOID: f32 = -10.0;
const Z_CORRIDOR: f32 = -9.0;
const Z_WIN_ZONE: f32 = -8.5;

/// Create a pentagram (5-pointed star) inside a circle, mirrored horizontally (point down)
fn create_pentagram_in_circle_mesh(radius: f32, line_thickness: f32) -> Mesh {
    use std::f32::consts::PI;

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Outer circle
    let circle_segments = 32;
    for i in 0..circle_segments {
        let angle1 = (i as f32) * 2.0 * PI / circle_segments as f32;
        let angle2 = ((i + 1) as f32) * 2.0 * PI / circle_segments as f32;

        let inner_r = radius - line_thickness;
        let outer_r = radius;

        let base_idx = positions.len() as u32;

        positions.push([inner_r * angle1.cos(), inner_r * angle1.sin(), 0.0]);
        positions.push([outer_r * angle1.cos(), outer_r * angle1.sin(), 0.0]);
        positions.push([outer_r * angle2.cos(), outer_r * angle2.sin(), 0.0]);
        positions.push([inner_r * angle2.cos(), inner_r * angle2.sin(), 0.0]);

        indices.extend_from_slice(&[
            base_idx, base_idx + 1, base_idx + 2,
            base_idx, base_idx + 2, base_idx + 3,
        ]);
    }

    // 5 outer points of the star, mirrored (point facing down instead of up)
    let star_radius = radius * 0.85;
    let mut outer_points = Vec::new();
    for i in 0..5 {
        // PI/2 instead of -PI/2 to flip vertically (point down)
        let angle = PI / 2.0 + (i as f32) * 2.0 * PI / 5.0;
        outer_points.push(Vec2::new(angle.cos() * star_radius, angle.sin() * star_radius));
    }

    // Pentagram connects every other point: 0->2->4->1->3->0
    let star_order = [0, 2, 4, 1, 3, 0];

    // Create line segments with thickness
    for i in 0..5 {
        let p1 = outer_points[star_order[i]];
        let p2 = outer_points[star_order[i + 1]];

        let dir = (p2 - p1).normalize();
        let perp = Vec2::new(-dir.y, dir.x) * line_thickness;

        let base_idx = positions.len() as u32;

        positions.push([p1.x - perp.x, p1.y - perp.y, 0.0]);
        positions.push([p1.x + perp.x, p1.y + perp.y, 0.0]);
        positions.push([p2.x + perp.x, p2.y + perp.y, 0.0]);
        positions.push([p2.x - perp.x, p2.y - perp.y, 0.0]);

        indices.extend_from_slice(&[
            base_idx, base_idx + 1, base_idx + 2,
            base_idx, base_idx + 2, base_idx + 3,
        ]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn spawn_level_background(
    commands: &mut Commands,
    level: &LevelData,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let bounds = &level.bounds;

    let void_color = Color::srgb(0.05, 0.05, 0.08);
    let corridor_color = Color::srgb(0.15, 0.15, 0.18);

    let void_size = Vec2::new(2000.0, 2000.0);
    let void_mesh = meshes.add(Rectangle::new(void_size.x, void_size.y));
    let void_material = materials.add(void_color);

    commands.spawn((
        VoidBackground,
        Mesh2d(void_mesh),
        MeshMaterial2d(void_material),
        Transform::from_xyz(bounds.center().x, bounds.center().y, Z_VOID),
    ));

    for walkable in &level.walkable {
        let width = walkable.max.x - walkable.min.x;
        let height = walkable.max.y - walkable.min.y;
        let center_x = (walkable.min.x + walkable.max.x) / 2.0;
        let center_y = (walkable.min.y + walkable.max.y) / 2.0;

        let corridor_mesh = meshes.add(Rectangle::new(width, height));
        let corridor_material = materials.add(corridor_color);

        commands.spawn((
            LevelBackground,
            Mesh2d(corridor_mesh),
            MeshMaterial2d(corridor_material),
            Transform::from_xyz(center_x, center_y, Z_CORRIDOR),
        ));
    }
}

pub fn spawn_win_zone(
    commands: &mut Commands,
    position: Vec2,
    radius: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    // Pentagram inside circle
    let zone_mesh = meshes.add(create_pentagram_in_circle_mesh(radius, 0.8));
    let zone_material = materials.add(Color::srgba(0.8, 0.7, 0.2, 0.6));

    commands.spawn((
        WinZone { radius },
        Mesh2d(zone_mesh),
        MeshMaterial2d(zone_material),
        Transform::from_xyz(position.x, position.y, Z_WIN_ZONE),
    )).with_children(|parent| {
        // Timer text inside pentagram (smaller, centered)
        parent.spawn((
            WinZoneTimerText,
            Text2d::new(""),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgba(1.0, 1.0, 0.5, 0.9)),
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
    });
}
