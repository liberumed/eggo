// Re-export from new domain modules
pub use crate::combat::components::*;
pub use crate::combat::weapons::*;
// EquippedWeaponId moved to inventory to avoid circular deps
pub use crate::inventory::components::EquippedWeaponId;
