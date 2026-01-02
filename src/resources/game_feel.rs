use bevy::prelude::*;

/// Hitstop - brief freeze when hitting enemies
#[derive(Resource, Default)]
pub struct Hitstop {
    pub timer: f32,
}

impl Hitstop {
    pub fn trigger(&mut self, duration: f32) {
        self.timer = duration;
    }

    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }

    pub fn tick(&mut self, dt: f32) {
        self.timer = (self.timer - dt).max(0.0);
    }
}

/// Screen shake effect
#[derive(Resource, Default)]
pub struct ScreenShake {
    pub intensity: f32,
    pub timer: f32,
}

impl ScreenShake {
    pub fn trigger(&mut self, intensity: f32, duration: f32) {
        self.intensity = intensity;
        self.timer = duration;
    }

    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }

    pub fn tick(&mut self, dt: f32) {
        self.timer = (self.timer - dt).max(0.0);
    }

    /// Get current shake offset (random within intensity, decays over time)
    pub fn get_offset(&self) -> Vec2 {
        if !self.is_active() {
            return Vec2::ZERO;
        }
        let decay = self.timer / 0.2; // Assumes max duration ~0.2s
        let decay = decay.min(1.0);
        let range = self.intensity * decay;
        Vec2::new(
            (rand::random::<f32>() - 0.5) * 2.0 * range,
            (rand::random::<f32>() - 0.5) * 2.0 * range,
        )
    }
}
