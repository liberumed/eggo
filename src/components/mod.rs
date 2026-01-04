mod inventory;
mod ui;

pub use inventory::*;
pub use ui::*;

// Re-export from player for backwards compatibility
pub use crate::player::{
    Player, PlayerAnimation, Dashing, Sprinting, PlayerAttackState,
    SpriteAnimation,
};

// Re-export from creatures for backwards compatibility
pub use crate::creatures::{
    Creature, CreatureAnimation, Hostile, Glowing,
    CreatureDefinition, creature_catalog,
};

// Re-export from core for backwards compatibility
pub use crate::core::{
    Health, Dead, Stunned, Knockback, Blocking, DespawnTimer, Shadow, DeathAnimation, Loot,
    StaticCollider, WalkCollider, HitCollider, YSorted,
};

// Re-export from combat for backwards compatibility
pub use crate::combat::{
    Fist, WeaponVisual, WeaponVisualMesh, WeaponRangeIndicator,
    PlayerRangeIndicator, CreatureRangeIndicator, WeaponSwing, AttackType,
    Rarity, Weapon, weapon_catalog, Equipment,
    Drawn, PlayerWeapon,
};
