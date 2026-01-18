#![allow(dead_code)]

use bevy::prelude::*;

use crate::core::{Knockback, Stunned};
use super::super::Rarity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum AttackType {
    #[default]
    Slash,
    Stab,
    Smash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DamageType {
    #[default]
    Physical,
}

/// Effect applied when a weapon hits a target
#[derive(Clone)]
pub enum OnHitEffect {
    Stun { duration: f32 },
    Knockback { force: f32 },
}

impl OnHitEffect {
    pub fn apply(&self, commands: &mut Commands, entity: Entity, direction: Vec2) {
        match self {
            OnHitEffect::Stun { duration } => {
                commands.entity(entity).insert(Stunned(*duration));
            }
            OnHitEffect::Knockback { force } => {
                commands.entity(entity).insert(Knockback {
                    velocity: direction * *force,
                    timer: 0.0,
                });
            }
        }
    }
}

/// Visual assets for weapon rendering
#[derive(Clone)]
pub struct WeaponVisual {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
    pub offset: f32,
}

/// Weapon definition and stats.
/// Tier values (1-5) scale to gameplay values via methods.
#[derive(Component, Clone)]
pub struct Weapon {
    pub name: String,
    pub visual: WeaponVisual,
    pub damage: i32,
    /// Attack speed tier: 1=slow, 5=fast
    pub speed: i32,
    /// Range tier: 1=short, 5=long
    pub reach: i32,
    /// Attack cone tier: 1=narrow, 5=wide
    pub arc: i32,
    pub attack_type: AttackType,
    pub damage_type: DamageType,
    pub rarity: Rarity,
    pub cost: u32,
    /// Block damage reduction tier
    pub block: i32,
    /// Block knockback reduction tier
    pub block_kb: i32,
    pub on_hit: Vec<OnHitEffect>,
}

impl Weapon {
    pub fn attack_speed(&self) -> f32 {
        1.0 + self.speed as f32 * 0.5
    }

    pub fn swing_duration(&self) -> f32 {
        1.0 / self.attack_speed()
    }

    pub fn range(&self) -> f32 {
        20.0 + self.reach as f32 * 10.0
    }

    pub fn cone_angle(&self) -> f32 {
        0.35 + self.arc as f32 * 0.25
    }

    pub fn block_damage_reduction(&self) -> f32 {
        0.1 + self.block as f32 * 0.15
    }

    pub fn block_knockback_reduction(&self) -> f32 {
        0.2 + self.block_kb as f32 * 0.15
    }

    pub fn apply_on_hit(&self, commands: &mut Commands, entity: Entity, direction: Vec2) {
        for effect in &self.on_hit {
            effect.apply(commands, entity, direction);
        }
    }

    pub fn knockback_force(&self) -> f32 {
        self.on_hit
            .iter()
            .filter_map(|e| match e {
                OnHitEffect::Knockback { force } => Some(*force),
                _ => None,
            })
            .sum()
    }
}

pub mod weapon_catalog {
    use super::*;

    pub fn wooden_stick(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Weapon {
        Weapon {
            name: "Wooden Stick".to_string(),
            visual: WeaponVisual {
                mesh: meshes.add(Rectangle::new(18.0, 2.5)),
                material: materials.add(Color::srgb(0.55, 0.4, 0.25)),
                offset: 12.0,
            },
            damage: 1,
            speed: 1,
            reach: 3,
            arc: 2,
            attack_type: AttackType::Smash,
            damage_type: DamageType::Physical,
            rarity: Rarity::Common,
            cost: 0,
            block: 1,
            block_kb: 1,
            on_hit: vec![
                OnHitEffect::Knockback { force: 330.0 },
            ],
        }
    }

    pub fn rusty_knife(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Weapon {
        Weapon {
            name: "Rusty Knife".to_string(),
            visual: WeaponVisual {
                mesh: meshes.add(Triangle2d::new(
                    Vec2::new(0.0, 2.0),
                    Vec2::new(0.0, -2.0),
                    Vec2::new(12.0, 0.0),
                )),
                material: materials.add(Color::srgb(0.75, 0.75, 0.8)),
                offset: 14.0,
            },
            damage: 2,
            speed: 4,
            reach: 2,
            arc: 1,
            attack_type: AttackType::Slash,
            damage_type: DamageType::Physical,
            rarity: Rarity::Common,
            cost: 10,
            block: 2,
            block_kb: 3,
            on_hit: vec![
                OnHitEffect::Stun { duration: 0.65 },
                OnHitEffect::Knockback { force: 70.0 },
            ],
        }
    }

    /// Fast sword for satisfying combo attacks (uses sprite animations)
    pub fn sword(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Weapon {
        Weapon {
            name: "Sword".to_string(),
            visual: WeaponVisual {
                mesh: meshes.add(Circle::new(1.0)),  // Minimal mesh (not displayed for Smash)
                material: materials.add(Color::NONE),
                offset: 0.0,
            },
            damage: 2,
            speed: 5,      // Fast swings for good combo feel
            reach: 3,      // Medium-long reach
            arc: 2,        // Medium arc
            attack_type: AttackType::Smash,  // Uses sprite animation for combos
            damage_type: DamageType::Physical,
            rarity: Rarity::Uncommon,
            cost: 25,
            block: 3,
            block_kb: 2,
            on_hit: vec![
                OnHitEffect::Stun { duration: 0.3 },
                OnHitEffect::Knockback { force: 120.0 },
            ],
        }
    }

    pub fn fist(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Weapon {
        Weapon {
            name: "Fist".to_string(),
            visual: WeaponVisual {
                mesh: meshes.add(Circle::new(3.0)),
                material: materials.add(Color::srgb(0.8, 0.65, 0.5)),
                offset: 11.0,
            },
            damage: 1,
            speed: 2,
            reach: 1,
            arc: 1,
            attack_type: AttackType::Smash,
            damage_type: DamageType::Physical,
            rarity: Rarity::Common,
            cost: 0,
            block: 1,
            block_kb: 1,
            on_hit: vec![
                OnHitEffect::Stun { duration: 0.30 },
                OnHitEffect::Knockback { force: 100.0 },
            ],
        }
    }

    /// Club weapon for Goblins - slower but stronger than fist
    pub fn club(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Weapon {
        Weapon {
            name: "Club".to_string(),
            visual: WeaponVisual {
                mesh: meshes.add(Capsule2d::new(3.0, 12.0)),
                material: materials.add(Color::srgb(0.5, 0.35, 0.2)),
                offset: 14.0,
            },
            damage: 2,
            speed: 2,
            reach: 2,
            arc: 2,
            attack_type: AttackType::Smash,
            damage_type: DamageType::Physical,
            rarity: Rarity::Common,
            cost: 0,
            block: 1,
            block_kb: 1,
            on_hit: vec![
                OnHitEffect::Knockback { force: 150.0 },
            ],
        }
    }
}
