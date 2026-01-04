use bevy::prelude::*;

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
