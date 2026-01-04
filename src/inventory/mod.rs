pub mod components;
pub mod data;

pub use components::*;
pub use data::*;

// Note: systems.rs and ui.rs will be created during cleanup phase
// by extracting from systems/inventory.rs and components/ui.rs
// The InventoryPlugin will also be added then
