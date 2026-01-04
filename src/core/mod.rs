pub mod camera;
pub mod collisions;
pub mod components;
pub mod depth;
pub mod input;
pub mod state;
pub mod systems;

pub use collisions::*;
pub use components::*;
pub use depth::YSorted;
pub use input::*;
pub use state::*;

use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<InputBindings>()
            .init_resource::<WorldConfig>()
            .init_resource::<NewGameRequested>()
            .add_systems(Update, (
                systems::update_stun,
                systems::update_despawn_timer,
                camera::camera_follow,
            ))
            .add_systems(
                PostUpdate,
                depth::update_y_depth.run_if(in_state(GameState::Playing)),
            );
    }
}
