use bevy::prelude::*;

use crate::combat::{AttackType, Weapon};
use crate::constants::ATTACK_HIT_DELAY_PERCENT;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AttackPhase {
    #[default]
    WindUp,
    Strike,
    Recovery,
}

#[derive(Component, Clone, Debug)]
pub struct AttackTimer {
    pub elapsed: f32,
    pub wind_up_duration: f32,
    pub strike_duration: f32,
    pub recovery_duration: f32,

    pub hit_applied: bool,
    pub target_direction: Vec2,

    pub base_angle: Option<f32>,
    pub attack_type: AttackType,
}

impl AttackTimer {
    pub fn from_weapon(weapon: &Weapon, target_dir: Vec2) -> Self {
        let total = weapon.swing_duration();
        Self {
            elapsed: 0.0,
            wind_up_duration: total * ATTACK_HIT_DELAY_PERCENT,
            strike_duration: 0.05,
            recovery_duration: total * (1.0 - ATTACK_HIT_DELAY_PERCENT) - 0.05,
            hit_applied: false,
            target_direction: target_dir,
            base_angle: Some(target_dir.y.atan2(target_dir.x)),
            attack_type: weapon.attack_type,
        }
    }

    pub fn current_phase(&self) -> AttackPhase {
        if self.elapsed < self.wind_up_duration {
            AttackPhase::WindUp
        } else if self.elapsed < self.wind_up_duration + self.strike_duration {
            AttackPhase::Strike
        } else {
            AttackPhase::Recovery
        }
    }

    pub fn total_duration(&self) -> f32 {
        self.wind_up_duration + self.strike_duration + self.recovery_duration
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.total_duration()
    }

    pub fn progress(&self) -> f32 {
        self.elapsed / self.total_duration()
    }

    pub fn tick(&mut self, delta: f32) {
        self.elapsed += delta;
    }
}
