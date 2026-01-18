use bevy::prelude::*;

use crate::constants::{CARDINAL_DOWN, CARDINAL_LEFT, CARDINAL_RIGHT, CARDINAL_UP};
use crate::core::HitCollider;

/// Precomputed cone for efficient hit testing.
/// Create once, test against many targets without expensive trig per-target.
pub struct HitCone {
    pub origin: Vec2,
    pub direction: Vec2,
    pub range: f32,
    cos_half: f32,
    sin_half: f32,
}

impl HitCone {
    /// Create cone with precomputed trig values.
    /// `full_angle` is the total cone angle in radians (not half-angle).
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

    /// Check if a single circle intersects the cone.
    fn hits_circle(&self, center: Vec2, radius: f32) -> bool {
        let to_target = center - self.origin;
        let distance = to_target.length();

        if distance < 0.001 {
            return false;
        }

        let in_range = distance - radius < self.range;

        let dot = to_target.dot(self.direction);
        let in_cone = dot > distance * self.cos_half - radius * self.sin_half;

        in_range && in_cone
    }

    /// Check if any circle in the compound collider intersects the cone.
    pub fn hits_collider(&self, entity_pos: Vec2, collider: &HitCollider) -> bool {
        for circle in &collider.circles {
            let center = entity_pos + circle.offset;
            if self.hits_circle(center, circle.radius) {
                return true;
            }
        }
        false
    }

    /// Check if target hitbox intersects cone (single circle, for simple cases).
    pub fn hits(&self, target_pos: Vec2, target_radius: f32) -> bool {
        self.hits_circle(target_pos, target_radius)
    }
}

/// Convert angle (radians) to unit direction vector.
/// Cheaper than calling cos/sin separately when you need both.
#[inline]
pub fn angle_to_direction(angle: f32) -> Vec2 {
    Vec2::new(angle.cos(), angle.sin())
}

/// Snap angle to nearest cardinal direction (Right, Up, Left, Down)
pub fn snap_to_cardinal(angle: f32) -> f32 {
    use std::f32::consts::{FRAC_PI_4, TAU};

    // Normalize to [0, 2π)
    let normalized = angle.rem_euclid(TAU);

    // Determine quadrant (each cardinal owns 90° around it)
    if normalized < FRAC_PI_4 || normalized >= 7.0 * FRAC_PI_4 {
        CARDINAL_RIGHT  // 0°
    } else if normalized < 3.0 * FRAC_PI_4 {
        CARDINAL_UP     // 90°
    } else if normalized < 5.0 * FRAC_PI_4 {
        CARDINAL_LEFT   // 180°
    } else {
        CARDINAL_DOWN   // 270° (-90°)
    }
}
