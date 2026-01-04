pub mod components;
pub mod sprites;
pub mod stats;

pub use components::*;
pub use sprites::*;
pub use stats::*;

// Note: spawner.rs and systems.rs will be created during cleanup phase
// by extracting from spawners/character.rs and systems/movement.rs
// The PlayerPlugin will also be added then
