use bevy::prelude::*;

/// Player marker component
#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct PlayerAnimation {
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

/// Active attack state for sprite-based attacks (Smash weapons)
#[derive(Component)]
pub struct PlayerAttackState {
    pub facing_right: bool,
    pub timer: f32,
    pub duration: f32,
    pub hit_applied: bool,
}

/// Component for animated sprites
#[derive(Component)]
pub struct SpriteAnimation {
    pub current_animation: String,
    pub frame_index: usize,
    pub timer: Timer,
    pub flip_x: bool,
    pub speed: f32,
    pub animation_changed: bool,
}

impl SpriteAnimation {
    pub fn new(animation: &str, frame_duration_ms: u32) -> Self {
        Self {
            current_animation: animation.to_string(),
            frame_index: 0,
            timer: Timer::from_seconds(frame_duration_ms as f32 / 1000.0, TimerMode::Repeating),
            flip_x: false,
            speed: 1.0,
            animation_changed: false,
        }
    }

    pub fn set_animation(&mut self, animation: &str) {
        if self.current_animation != animation {
            self.current_animation = animation.to_string();
            self.frame_index = 0;
            self.timer.reset();
            self.animation_changed = true;
        }
    }
}
