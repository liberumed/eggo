use bevy::prelude::*;

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
                show_death_screen,
                handle_new_game_button,
            ),
        );
    }
}
