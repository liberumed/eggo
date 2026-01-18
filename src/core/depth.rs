use bevy::prelude::*;

use crate::levels::CurrentLevel;
use super::collisions::{StaticCollider, WalkCollider, YSorted};
use super::components::Dead;

const Z_MIN: f32 = -1.0;
const Z_MAX: f32 = 1.0;
const DEFAULT_Y_MIN: f32 = -600.0;
const DEFAULT_Y_MAX: f32 = 600.0;

fn y_to_z(y: f32, y_min: f32, y_max: f32) -> f32 {
    let y_clamped = y.clamp(y_min, y_max);
    let t = (y_clamped - y_min) / (y_max - y_min);
    Z_MAX - t * (Z_MAX - Z_MIN)
}

/// Updates Z depth based on ground contact point (from colliders, YSorted as fallback)
pub fn update_y_depth(
    level: Res<CurrentLevel>,
    mut query: Query<
        (&mut Transform, &YSorted, Option<&WalkCollider>, Option<&StaticCollider>),
        Without<Dead>,
    >,
) {
    let (y_min, y_max) = level
        .bounds()
        .map(|b| (b.min.y, b.max.y))
        .unwrap_or((DEFAULT_Y_MIN, DEFAULT_Y_MAX));

    for (mut transform, y_sorted, walk_collider, static_collider) in &mut query {
        let base_offset = walk_collider
            .map(|w| w.offset_y)
            .or_else(|| static_collider.map(|s| s.offset_y))
            .unwrap_or(y_sorted.base_offset);

        let base_y = transform.translation.y + base_offset;
        transform.translation.z = y_to_z(base_y, y_min, y_max);
    }
}
