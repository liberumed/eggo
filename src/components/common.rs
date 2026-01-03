use bevy::prelude::*;

#[derive(Component)]
pub struct Health(pub i32);

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct Stunned(pub f32);

#[derive(Component)]
pub struct Knockback {
    pub velocity: Vec2,
    pub timer: f32,
}

#[derive(Component)]
pub struct Blocking;

#[derive(Component)]
pub struct DespawnTimer(pub f32);

/// Shadow component for characters
#[derive(Component)]
pub struct Shadow {
    pub base_scale: Vec2,
}

impl Default for Shadow {
    fn default() -> Self {
        Self { base_scale: Vec2::ONE }
    }
}

#[derive(Component)]
pub struct DeathAnimation {
    pub timer: f32,
    pub stage: u8,
}

#[derive(Component, Clone, Copy)]
pub struct Loot {
    pub philosophy: bool,
    pub nature_study: bool,
    pub wisdom: bool,
}

/// Walk collision for static props (ellipse at base)
#[derive(Component)]
pub struct StaticCollider {
    pub radius_x: f32,
    pub radius_y: f32,
    pub offset_y: f32,
}

/// Walk collision for characters (ellipse at base)
#[derive(Component)]
pub struct WalkCollider {
    pub radius_x: f32,
    pub radius_y: f32,
    pub offset_y: f32,
}

/// Hit collision (hurtbox) - used for taking damage (ellipse)
#[derive(Component)]
pub struct HitCollider {
    pub radius_x: f32,
    pub radius_y: f32,
    pub offset_y: f32,
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
