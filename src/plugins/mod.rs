mod depth;
mod effects;
mod inventory;
mod status;
mod ui;

pub use depth::DepthPlugin;
pub use effects::EffectsPlugin;
pub use inventory::InventoryPlugin;
pub use status::StatusPlugin;
pub use ui::UiPlugin;

// Re-export from creatures for backwards compatibility
pub use crate::creatures::CreaturePlugin;

// Re-export from player for backwards compatibility
pub use crate::player::PlayerPlugin;
