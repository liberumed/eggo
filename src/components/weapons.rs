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

#[derive(Component)]
pub struct Weapon {
    pub name: String,
    pub attack_speed: f32,
    pub damage: i32,
    pub range: f32,
    pub cone_angle: f32,
    pub knockback: f32,
    pub attack_type: AttackType,
    pub damage_type: DamageType,
    pub rarity: Rarity,
    pub cost: u32,
}

impl Weapon {
    pub fn swing_duration(&self) -> f32 {
        1.0 / self.attack_speed
    }
}

pub const UNARMED_STATS: Weapon = Weapon {
    name: String::new(),
    attack_speed: 2.5,
    damage: 1,
    range: 30.0,
    cone_angle: 1.57,
    knockback: 50.0,
    attack_type: AttackType::Smash,
    damage_type: DamageType::Physical,
    rarity: Rarity::Common,
    cost: 0,
};

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
