pub mod components;
pub mod systems;

pub use components::*;
pub use systems::*;

use bevy::prelude::*;

use crate::core::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_counters,
                update_hp_text,
                update_weapon_info,
                stabilize_text_rotation,
                stabilize_shadow,
            ),
        )
        .add_systems(
            Update,
            show_death_menu.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                handle_resume_button,
                handle_menu_new_game_button,
                handle_exit_button,
            ),
        );
    }
}
