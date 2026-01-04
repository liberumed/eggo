use bevy::prelude::*;

use super::collisions::{StaticCollider, WalkCollider, YSorted};
use super::components::Dead;

const Y_MIN: f32 = -600.0;
const Y_MAX: f32 = 600.0;
const Z_MIN: f32 = -1.0;
const Z_MAX: f32 = 1.0;

fn y_to_z(y: f32) -> f32 {
    let y_clamped = y.clamp(Y_MIN, Y_MAX);
    let t = (y_clamped - Y_MIN) / (Y_MAX - Y_MIN);
    Z_MAX - t * (Z_MAX - Z_MIN)
}

/// Updates Z depth based on ground contact point (from colliders, YSorted as fallback)
pub fn update_y_depth(
    mut query: Query<
        (&mut Transform, &YSorted, Option<&WalkCollider>, Option<&StaticCollider>),
        Without<Dead>,
    >,
) {
    for (mut transform, y_sorted, walk_collider, static_collider) in &mut query {
        // Use collider offset as ground contact point, fall back to YSorted.base_offset
        let base_offset = walk_collider
            .map(|w| w.offset_y)
            .or_else(|| static_collider.map(|s| s.offset_y))
            .unwrap_or(y_sorted.base_offset);

        let base_y = transform.translation.y + base_offset;
        transform.translation.z = y_to_z(base_y);
    }
}
