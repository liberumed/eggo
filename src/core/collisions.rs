use bevy::prelude::*;

/// Walk collision for static props (ellipse at base)
#[derive(Component)]
pub struct StaticCollider {
    pub radius_x: f32,
    pub radius_y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

/// Walk collision for characters (ellipse at base)
#[derive(Component)]
pub struct WalkCollider {
    pub radius_x: f32,
    pub radius_y: f32,
    pub offset_y: f32,
}

/// Single circle in a compound hit collider
#[derive(Clone, Debug)]
pub struct HitCircle {
    pub offset: Vec2,   // Relative to entity position
    pub radius: f32,
}

impl HitCircle {
    pub fn new(offset_x: f32, offset_y: f32, radius: f32) -> Self {
        Self { offset: Vec2::new(offset_x, offset_y), radius }
    }
}

/// Hit collision (hurtbox) - compound collider with multiple circles
/// Allows accurate hit detection for any shape (ellipse approximation, boss body parts, etc.)
#[derive(Component, Clone)]
pub struct HitCollider {
    pub circles: Vec<HitCircle>,
}

impl HitCollider {
    /// Create from a list of circles
    pub fn new(circles: Vec<HitCircle>) -> Self {
        Self { circles }
    }

    /// Convenience: single circle collider
    pub fn circle(offset_x: f32, offset_y: f32, radius: f32) -> Self {
        Self { circles: vec![HitCircle::new(offset_x, offset_y, radius)] }
    }

    /// Convenience: approximate ellipse with 2 overlapping circles (vertical ellipse)
    pub fn ellipse_vertical(offset_y: f32, radius_x: f32, radius_y: f32) -> Self {
        let r = radius_x;
        let stretch = radius_y - radius_x;
        if stretch <= 0.0 {
            return Self::circle(0.0, offset_y, radius_x);
        }
        Self {
            circles: vec![
                HitCircle::new(0.0, offset_y - stretch * 0.5, r),
                HitCircle::new(0.0, offset_y + stretch * 0.5, r),
            ],
        }
    }

    /// Get bounding radius for broad-phase checks (includes offset distance)
    pub fn bounding_radius(&self) -> f32 {
        self.circles.iter()
            .map(|c| c.offset.length() + c.radius)
            .fold(0.0_f32, |a, b| a.max(b))
    }

    /// Get largest circle radius (for range checks - offset doesn't extend attack range)
    pub fn max_radius(&self) -> f32 {
        self.circles.iter()
            .map(|c| c.radius)
            .fold(0.0_f32, |a, b| a.max(b))
    }

    /// Get max offset length (how far circles are from entity center)
    pub fn max_offset(&self) -> f32 {
        self.circles.iter()
            .map(|c| c.offset.length())
            .fold(0.0_f32, |a, b| a.max(b))
    }
}

/// Check if two ellipses overlap (simplified axis-aligned)
pub fn ellipses_overlap(
    pos_a: Vec2,
    radius_a: Vec2,  // (radius_x, radius_y)
    pos_b: Vec2,
    radius_b: Vec2,
) -> bool {
    let diff = pos_a - pos_b;
    let combined = radius_a + radius_b;
    // Normalized distance check
    let nx = diff.x / combined.x;
    let ny = diff.y / combined.y;
    nx * nx + ny * ny < 1.0
}

/// Get push vector to separate two overlapping ellipses
pub fn ellipse_push(
    pos_a: Vec2,
    radius_a: Vec2,
    pos_b: Vec2,
    radius_b: Vec2,
) -> Vec2 {
    let diff = pos_a - pos_b;
    let combined = radius_a + radius_b;

    if diff.length_squared() < 0.001 {
        return Vec2::new(combined.x, 0.0);
    }

    // Scale to unit circle space
    let scaled_diff = Vec2::new(diff.x / combined.x, diff.y / combined.y);
    let scaled_dist = scaled_diff.length();

    if scaled_dist >= 1.0 {
        return Vec2::ZERO;
    }

    // Push direction in scaled space, then scale back
    let push_dir = scaled_diff.normalize();
    let overlap = 1.0 - scaled_dist;

    Vec2::new(
        push_dir.x * overlap * combined.x,
        push_dir.y * overlap * combined.y,
    )
}

#[derive(Component)]
pub struct YSorted {
    pub base_offset: f32,
}
