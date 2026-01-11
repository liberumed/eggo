use bevy::prelude::*;

/// Marker for weapon range indicator (arc at attack range)
#[derive(Component)]
pub struct WeaponRangeIndicator;

/// Marker for player's range indicator (to distinguish from creature indicators)
#[derive(Component)]
pub struct PlayerRangeIndicator;

/// Links a creature's range indicator to its owner
#[derive(Component)]
pub struct CreatureRangeIndicator(pub Entity);

/// Links a goblin's attack area indicator (filled half-circle) to its owner
#[derive(Component)]
pub struct GoblinAttackIndicator(pub Entity);

/// Equipment slots on an entity
#[allow(dead_code)]
#[derive(Component, Default)]
pub struct Equipment {
    pub main_hand: Option<Entity>,
    pub chest: Option<Entity>,
}

/// Marks an item as equippable in a slot
#[allow(dead_code)]
#[derive(Component)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    MainHand,
    Chest,
}
