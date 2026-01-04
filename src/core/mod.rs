pub mod camera;
pub mod collisions;
pub mod components;
pub mod depth;
pub mod input;
pub mod state;
pub mod systems;

pub use camera::*;
pub use collisions::*;
pub use components::*;
pub use depth::*;
pub use input::*;
pub use state::*;
pub use systems::*;

use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_follow, update_stun, update_despawn_timer))
            .add_systems(
                PostUpdate,
                update_y_depth.run_if(in_state(GameState::Playing)),
            );
    }
}
