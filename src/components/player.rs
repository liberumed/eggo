use bevy::prelude::*;

/// Player marker component
#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct PlayerAnimation {
    pub time: f32,
    pub velocity: Vec2,
}

/// Active dash state
#[derive(Component)]
pub struct Dashing {
    pub direction: Vec2,
    pub timer: f32,
}

/// Dash cooldown tracker
#[derive(Component, Default)]
pub struct DashCooldown {
    pub timer: f32,
}

/// Sprint state tracker for speed ramp-up
#[derive(Component, Default)]
pub struct Sprinting {
    pub duration: f32,
}

/// Brief phase-through after dash (prevents getting stuck in creatures)
#[derive(Component)]
pub struct PhaseThrough {
    pub timer: f32,
}
