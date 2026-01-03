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
