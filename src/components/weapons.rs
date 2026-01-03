#![allow(dead_code)]

use bevy::prelude::*;

use super::{Knockback, Stunned};

use super::ItemId;

#[derive(Component)]
pub struct Knife;

#[derive(Component)]
pub struct Stick;

#[derive(Component)]
pub struct Fist;

/// Tracks which weapon ItemId is currently equipped
#[derive(Component)]
pub struct EquippedWeaponId(pub ItemId);

/// Visual assets for weapon rendering - stores actual asset handles
#[derive(Clone)]
pub struct WeaponVisual {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
    pub offset: f32,
}

/// Marker for weapon visual mesh children - used to find and despawn when swapping
#[derive(Component)]
pub struct WeaponVisualMesh;

#[derive(Component)]
pub struct WeaponSwing {
    pub timer: f32,
    pub duration: f32,
    pub base_angle: Option<f32>,
    pub attack_type: AttackType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Effect applied when a weapon hits a target
#[derive(Clone)]
pub enum OnHitEffect {
    Stun { duration: f32 },
    Knockback { force: f32 },
}

impl OnHitEffect {
    /// Apply this effect to an entity
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

/// Weapon stats use tier values (1-5) that scale to gameplay values via methods.
#[derive(Component, Clone)]
pub struct Weapon {
    pub name: String,
    /// Visual appearance of the weapon
    pub visual: WeaponVisual,
    /// Direct damage value per hit
    pub damage: i32,
    /// Attack speed tier: 1=slow, 5=fast → attack_speed() = 1.0 + speed * 0.5
    pub speed: i32,
    /// Range tier: 1=short, 5=long → range() = 20 + reach * 10
    pub reach: i32,
    /// Attack cone tier: 1=narrow, 5=wide → cone_angle() = 0.35 + arc * 0.25 rad
    pub arc: i32,
    /// Slash=wide arc, Stab=narrow/fast, Smash=slow/heavy
    pub attack_type: AttackType,
    /// Physical, Fire, Ice, etc. (for future resistances)
    pub damage_type: DamageType,
    /// Common to Legendary (affects loot/visuals)
    pub rarity: Rarity,
    /// Buy/sell value
    pub cost: u32,
    /// Block damage reduction tier: 1=weak, 5=strong → block_damage_reduction() = 0.1 + block * 0.15
    pub block: i32,
    /// Block knockback reduction tier: 1=weak, 5=strong → block_knockback_reduction() = 0.2 + block_kb * 0.15
    pub block_kb: i32,
    /// Effects applied when this weapon hits a target
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

    /// Apply all on-hit effects to a target entity
    pub fn apply_on_hit(&self, commands: &mut Commands, entity: Entity, direction: Vec2) {
        for effect in &self.on_hit {
            effect.apply(commands, entity, direction);
        }
    }

    /// Get the knockback force from on_hit effects (for blocking calculations)
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
                //OnHitEffect::Stun { duration: 0.7 },
                OnHitEffect::Knockback { force: 130.0 },
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
            arc: 2,
            attack_type: AttackType::Slash,
            damage_type: DamageType::Physical,
            rarity: Rarity::Common,
            cost: 10,
            block: 2,
            block_kb: 3,
            on_hit: vec![
                OnHitEffect::Stun { duration: 0.45 },
                OnHitEffect::Knockback { force: 20.0 },
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
            arc: 2,
            attack_type: AttackType::Smash,
            damage_type: DamageType::Physical,
            rarity: Rarity::Common,
            cost: 0,
            block: 1,
            block_kb: 1,
            on_hit: vec![
                OnHitEffect::Stun { duration: 0.15 },
                OnHitEffect::Knockback { force: 10.0 },
            ],
        }
    }
}


#[derive(Component, Default)]
pub struct Equipment {
    pub main_hand: Option<Entity>,
    pub chest: Option<Entity>,
}

#[derive(Component)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    MainHand,
    Chest,
}

#[derive(Component)]
pub struct Drawn;

#[derive(Component)]
pub struct PlayerWeapon;
