use bevy::prelude::*;

/// Component for animated sprites
#[derive(Component)]
pub struct SpriteAnimation {
    pub current_animation: String,
    pub frame_index: usize,
    pub timer: Timer,
    pub flip_x: bool,
    pub speed: f32,  // 1.0 = normal, 0.5 = half speed, 2.0 = double speed
}

impl SpriteAnimation {
    pub fn new(animation: &str, frame_duration_ms: u32) -> Self {
        Self {
            current_animation: animation.to_string(),
            frame_index: 0,
            timer: Timer::from_seconds(frame_duration_ms as f32 / 1000.0, TimerMode::Repeating),
            flip_x: false,
            speed: 1.0,
        }
    }

    pub fn set_animation(&mut self, animation: &str) {
        if self.current_animation != animation {
            self.current_animation = animation.to_string();
            self.frame_index = 0;
            self.timer.reset();
        }
    }
}
