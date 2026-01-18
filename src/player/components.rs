use bevy::prelude::*;

use crate::constants::{CAMERA_BASE_SCALE, CAMERA_ZOOM_OUT_SCALE, COMBO_TIMEOUT};

/// Player marker component
#[derive(Component)]
pub struct Player;

/// 4-directional facing for sprite animations
#[derive(Component, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum FacingDirection {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl FacingDirection {
    /// Get direction suffix for animation names (e.g., "_down", "_right")
    pub fn suffix(&self) -> &'static str {
        match self {
            FacingDirection::Up => "_up",
            FacingDirection::Down => "_down",
            FacingDirection::Left => "_left",
            FacingDirection::Right => "_right",
        }
    }

    /// Determine facing direction from a 2D direction vector
    pub fn from_vec2(dir: Vec2) -> Self {
        if dir.length_squared() < 0.01 {
            return FacingDirection::Down;
        }
        let abs_x = dir.x.abs();
        let abs_y = dir.y.abs();
        if abs_y > abs_x {
            if dir.y > 0.0 { FacingDirection::Up } else { FacingDirection::Down }
        } else {
            if dir.x > 0.0 { FacingDirection::Right } else { FacingDirection::Left }
        }
    }

    /// Determine facing direction from an angle (radians)
    pub fn from_angle(angle: f32) -> Self {
        use std::f32::consts::{FRAC_PI_4, PI};
        let angle = angle.rem_euclid(2.0 * PI);
        if angle < FRAC_PI_4 || angle >= 7.0 * FRAC_PI_4 {
            FacingDirection::Right
        } else if angle < 3.0 * FRAC_PI_4 {
            FacingDirection::Up
        } else if angle < 5.0 * FRAC_PI_4 {
            FacingDirection::Left
        } else {
            FacingDirection::Down
        }
    }
}

/// Tracks the current combo state for attack chains
#[derive(Component, Default)]
pub struct ComboState {
    /// Current attack in combo (0, 1, or 2 for attack 1/2/3)
    pub current_attack: u8,
    /// Time since the last attack completed
    pub time_since_attack: f32,
}

/// Brief hurt animation when taking damage
#[derive(Component)]
pub struct HurtAnimation {
    pub timer: f32,
    pub duration: f32,
}

impl Default for HurtAnimation {
    fn default() -> Self {
        Self {
            timer: 0.0,
            duration: 0.25,  // Short hurt flash
        }
    }
}

impl ComboState {
    /// Advance to the next attack in the combo chain
    pub fn advance(&mut self) {
        self.current_attack = (self.current_attack + 1) % 3;
        self.time_since_attack = 0.0;
    }

    /// Reset combo to the first attack
    pub fn reset(&mut self) {
        self.current_attack = 0;
        self.time_since_attack = 0.0;
    }

    /// Check if combo should be reset due to timeout
    pub fn should_reset(&self) -> bool {
        self.time_since_attack >= COMBO_TIMEOUT
    }

    /// Get the attack number (1, 2, or 3)
    pub fn attack_number(&self) -> u8 {
        self.current_attack + 1
    }
}

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

/// Raw movement input direction for current frame
#[derive(Component, Default)]
pub struct MovementInput(pub Vec2);

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
