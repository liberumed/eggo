mod animation;
mod inventory;
mod movement;
mod status;
mod ui;

pub use animation::*;
pub use inventory::*;
pub use movement::*;
pub use status::*;
pub use ui::*;

// Re-export from core for backwards compatibility
pub use crate::core::{camera_follow, update_y_depth};

// Combat systems are now in player/mod.rs plugin
