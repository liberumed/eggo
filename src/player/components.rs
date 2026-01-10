use bevy::prelude::*;

use crate::constants::{CAMERA_BASE_SCALE, CAMERA_ZOOM_OUT_SCALE};

/// Player marker component
#[derive(Component)]
pub struct Player;

/// Camera zoom state - zooms out when player is moving
#[derive(Resource)]
pub struct CameraState {
    pub current_scale: f32,
    pub target_scale: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            current_scale: CAMERA_BASE_SCALE,
            target_scale: CAMERA_BASE_SCALE,
        }
    }
}

impl CameraState {
    pub fn set_moving(&mut self, is_moving: bool) {
        self.target_scale = if is_moving {
            CAMERA_ZOOM_OUT_SCALE
        } else {
            CAMERA_BASE_SCALE
        };
    }
}

#[derive(Component, Default)]
pub struct PlayerAnimation {
    pub velocity: Vec2,
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
