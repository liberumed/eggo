use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};

use crate::constants::{ARC_SEGMENTS, ARC_THICKNESS};
use crate::inventory::Weapon;

/// Create arc mesh for a weapon's range indicator
pub fn create_weapon_arc(meshes: &mut Assets<Mesh>, weapon: &Weapon) -> Handle<Mesh> {
    meshes.add(create_arc_mesh(weapon.range(), weapon.cone_angle(), ARC_THICKNESS, ARC_SEGMENTS))
}

/// Create half-circle arc mesh for player attacks (180° = PI)
pub fn create_half_circle_arc(meshes: &mut Assets<Mesh>, range: f32) -> Handle<Mesh> {
    meshes.add(create_arc_mesh(range, std::f32::consts::PI, ARC_THICKNESS, ARC_SEGMENTS))
}

/// Create filled half-circle mesh for attack indicators (like debug mode)
/// Uses CircularSector - note: points +Y by default, needs -90° rotation offset
pub fn create_filled_half_circle(meshes: &mut Assets<Mesh>, range: f32) -> Handle<Mesh> {
    use bevy::math::primitives::CircularSector;
    // CircularSector takes half-angle, so PI/2 for a half-circle (180°)
    meshes.add(CircularSector::new(range, std::f32::consts::FRAC_PI_2))
}

/// Create an arc (annular sector) mesh - a thin ring segment
fn create_arc_mesh(range: f32, cone_angle: f32, thickness: f32, segments: u32) -> Mesh {
    let half_angle = cone_angle / 2.0;
    let inner_radius = (range - thickness).max(0.0);
    let outer_radius = range;

    let mut positions = Vec::with_capacity((segments as usize + 1) * 2);
    let mut indices = Vec::with_capacity(segments as usize * 6);

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let angle = -half_angle + t * cone_angle;
        let (sin_a, cos_a) = angle.sin_cos();

        // Inner vertex
        positions.push([inner_radius * cos_a, inner_radius * sin_a, 0.0]);
        // Outer vertex
        positions.push([outer_radius * cos_a, outer_radius * sin_a, 0.0]);
    }

    // Create triangles connecting inner and outer arcs
    for i in 0..segments {
        let base = i * 2;
        // Triangle 1
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 1);
        // Triangle 2
        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base + 3);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}
