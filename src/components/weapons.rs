use bevy::prelude::*;

#[derive(Component)]
pub struct Knife;

#[derive(Component)]
pub struct Fist;

#[derive(Component)]
pub struct WeaponSwing {
    pub timer: f32,
    pub duration: f32,
    pub base_angle: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttackType {
    #[default]
    Slash,
    Stab,
    Smash,
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

#[derive(Component)]
pub struct Weapon {
    pub name: String,
    pub damage: i32,
    pub speed: i32,
    pub reach: i32,
    pub arc: i32,
    pub impact: i32,
    pub attack_type: AttackType,
    pub rarity: Rarity,
}

impl Weapon {
    pub fn attack_speed(&self) -> f32 {
        1.0 + self.speed as f32 * 0.5
    }

    pub fn swing_duration(&self) -> f32 {
        1.0 / self.attack_speed()
    }

    pub fn range(&self) -> f32 {
        25.0 + self.reach as f32 * 12.0
    }

    pub fn cone_angle(&self) -> f32 {
        0.35 + self.arc as f32 * 0.25
    }

    pub fn knockback(&self) -> f32 {
        50.0 + self.impact as f32 * 40.0
    }
}

pub mod catalog {
    use super::*;

    pub fn rusty_knife() -> Weapon {
        Weapon {
            name: "Rusty Knife".to_string(),
            damage: 1,
            speed: 3,
            reach: 3,
            arc: 1,
            impact: 3,
            attack_type: AttackType::Slash,
            rarity: Rarity::Common,
        }
    }

    pub fn fist() -> Weapon {
        Weapon {
            name: "Fist".to_string(),
            damage: 1,
            speed: 4,
            reach: 1,
            arc: 5,
            impact: 2,
            attack_type: AttackType::Smash,
            rarity: Rarity::Common,
        }
    }
}


#[derive(Component, Default)]
pub struct Equipment {
    pub main_hand: Option<Entity>,
}

#[derive(Component)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    MainHand,
}

#[derive(Component)]
pub struct Drawn;

#[derive(Component)]
pub struct PlayerWeapon;
