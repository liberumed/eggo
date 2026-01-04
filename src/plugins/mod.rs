mod inventory;
mod ui;

pub use inventory::InventoryPlugin;
pub use ui::UiPlugin;

// Re-export from core for backwards compatibility
pub use crate::core::CorePlugin;

// Re-export from creatures for backwards compatibility
pub use crate::creatures::CreaturePlugin;

// Re-export from effects for backwards compatibility
pub use crate::effects::EffectsPlugin;

// Re-export from player for backwards compatibility
pub use crate::player::PlayerPlugin;
