use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};

use super::weapons::Weapon;

/// Precomputed cone for efficient hit testing
pub struct HitCone {
    pub origin: Vec2,
    pub direction: Vec2,
    pub range: f32,
    cos_half: f32,
    sin_half: f32,
}

impl HitCone {
    /// Create cone with precomputed trig values
    pub fn new(origin: Vec2, direction: Vec2, range: f32, full_angle: f32) -> Self {
        let half = full_angle / 2.0;
        Self {
            origin,
            direction,
            range,
            cos_half: half.cos(),
            sin_half: half.sin(),
        }
    }

    /// Check if target hitbox intersects cone
    pub fn hits(&self, target_pos: Vec2, target_radius: f32) -> bool {
        let to_target = target_pos - self.origin;
        let distance = to_target.length();

        if distance < 0.001 {
            return false;
        }

        let in_range = distance - target_radius < self.range;
        let dot = to_target.dot(self.direction);
        let in_cone = dot > distance * self.cos_half - target_radius * self.sin_half;

        in_range && in_cone
    }
}

/// Convert angle (radians) to unit direction vector
#[inline]
pub fn angle_to_direction(angle: f32) -> Vec2 {
    Vec2::new(angle.cos(), angle.sin())
}

const ARC_THICKNESS: f32 = 0.4;
const ARC_SEGMENTS: u32 = 16;

/// Create arc mesh for a weapon's range indicator
pub fn create_weapon_arc(meshes: &mut Assets<Mesh>, weapon: &Weapon) -> Handle<Mesh> {
    meshes.add(create_arc_mesh(weapon.range(), weapon.cone_angle(), ARC_THICKNESS, ARC_SEGMENTS))
}

/// Create an arc (annular sector) mesh
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

        positions.push([inner_radius * cos_a, inner_radius * sin_a, 0.0]);
        positions.push([outer_radius * cos_a, outer_radius * sin_a, 0.0]);
    }

    for i in 0..segments {
        let base = i * 2;
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 1);
        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base + 3);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices))
}
