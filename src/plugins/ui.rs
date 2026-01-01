use bevy::prelude::*;

use crate::resources::GameState;
use crate::systems::{
    handle_new_game_button, show_death_screen, stabilize_shadow, stabilize_text_rotation,
    update_counters, update_hp_text,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_counters,
                update_hp_text,
                stabilize_text_rotation,
                stabilize_shadow,
            ),
        )
        .add_systems(
            Update,
            show_death_screen.run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, handle_new_game_button);
    }
}
