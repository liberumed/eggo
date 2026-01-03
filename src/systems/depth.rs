use bevy::prelude::*;

use crate::components::{Dead, YSorted};

const Y_MIN: f32 = -600.0;
const Y_MAX: f32 = 600.0;
const Z_MIN: f32 = -1.0;
const Z_MAX: f32 = 1.0;

fn y_to_z(y: f32) -> f32 {
    let y_clamped = y.clamp(Y_MIN, Y_MAX);
    let t = (y_clamped - Y_MIN) / (Y_MAX - Y_MIN);
    Z_MAX - t * (Z_MAX - Z_MIN)
}

pub fn update_y_depth(
    mut query: Query<(&mut Transform, &YSorted), Without<Dead>>,
) {
    for (mut transform, y_sorted) in &mut query {
        let base_y = transform.translation.y + y_sorted.base_offset;
        transform.translation.z = y_to_z(base_y);
    }
}
