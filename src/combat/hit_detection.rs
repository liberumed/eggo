use bevy::prelude::*;

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

    /// Check if target hitbox intersects cone.
    /// Uses only: sqrt (for distance), dot product, multiply, subtract.
    /// No trig functions called - all precomputed in new().
    pub fn hits(&self, target_pos: Vec2, target_radius: f32) -> bool {
        let to_target = target_pos - self.origin;
        let distance = to_target.length();

        // Skip zero distance (target at origin)
        if distance < 0.001 {
            return false;
        }

        // Range check: hit if any part of hitbox is within range
        let in_range = distance - target_radius < self.range;

        // Cone check with hitbox expansion
        // Math derivation:
        //   Want: angle_to_target < half_angle + target_angular_size
        //   Where: target_angular_size ≈ atan(radius/distance) ≈ radius/distance (small angle)
        //   Using: cos(a+b) ≈ cos(a) - b*sin(a) for small b
        //   So: dot/distance > cos(half) - (radius/distance)*sin(half)
        //   Multiply both sides by distance to avoid division:
        //   dot > distance * cos_half - radius * sin_half
        let dot = to_target.dot(self.direction);
        let in_cone = dot > distance * self.cos_half - target_radius * self.sin_half;

        in_range && in_cone
    }

}

/// Convert angle (radians) to unit direction vector.
/// Cheaper than calling cos/sin separately when you need both.
#[inline]
pub fn angle_to_direction(angle: f32) -> Vec2 {
    Vec2::new(angle.cos(), angle.sin())
}

/// Cardinal direction angles in radians
pub const CARDINAL_RIGHT: f32 = 0.0;
pub const CARDINAL_UP: f32 = std::f32::consts::FRAC_PI_2;
pub const CARDINAL_LEFT: f32 = std::f32::consts::PI;
pub const CARDINAL_DOWN: f32 = -std::f32::consts::FRAC_PI_2;

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
