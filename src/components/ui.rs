use bevy::prelude::*;

#[derive(Component)]
pub struct DeathScreen;

#[derive(Component)]
pub struct NewGameButton;

#[derive(Component)]
pub struct HpText;

#[derive(Component)]
pub struct HeartSprite;

#[derive(Component)]
pub struct PhilosophyCounter;

#[derive(Component)]
pub struct NatureStudyCounter;

#[derive(Component)]
pub struct WisdomCounter;

#[derive(Component)]
pub struct WeaponInfoPanel;

#[derive(Component)]
pub struct WeaponNameText;

#[derive(Component)]
pub struct WeaponDamageText;

#[derive(Component)]
pub struct WeaponSpeedText;

#[derive(Component)]
pub struct WeaponRangeText;

#[derive(Component)]
pub struct WeaponConeText;

#[derive(Component)]
pub struct WeaponKnockbackText;

#[derive(Component)]
pub struct WeaponTypeText;

// Inventory UI components
#[derive(Component)]
pub struct HotbarUI;

#[derive(Component)]
pub struct HotbarSlot(pub usize);

#[derive(Component)]
pub struct HotbarSlotCount(pub usize);

#[derive(Component)]
pub struct InventoryPanel;

#[derive(Component)]
pub struct InventorySlotUI(pub usize);

#[derive(Component)]
pub struct InventorySlotCount(pub usize);
