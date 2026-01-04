pub mod components;
pub mod data;

pub use components::*;
pub use data::*;

// Note: spawner.rs and systems.rs will be created during cleanup phase
// by extracting from spawners/character.rs and systems/movement.rs
// The CreaturePlugin will also be added then
