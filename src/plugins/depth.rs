use bevy::prelude::*;

use crate::resources::GameState;
use crate::systems::update_y_depth;

pub struct DepthPlugin;

impl Plugin for DepthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_y_depth.run_if(in_state(GameState::Playing)),
        );
    }
}
