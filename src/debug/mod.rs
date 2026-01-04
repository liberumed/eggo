pub mod components;
pub mod config;
pub mod systems;

pub use components::*;
pub use config::*;

use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugConfig>()
            .add_systems(Update, (
                systems::toggle_collision_debug,
                systems::spawn_debug_circles,
                systems::spawn_weapon_debug_cones,
                systems::update_player_debug_cone,
                systems::update_creature_debug_circles,
                systems::update_debug_visibility,
            ));
    }
}
