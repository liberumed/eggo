pub mod components;
pub mod sprites;
pub mod stats;
pub mod systems;

pub use components::*;
pub use sprites::*;
pub use stats::*;
pub use systems::*;

use bevy::prelude::*;

use crate::core::GameState;
use crate::combat::{
    aim_weapon, toggle_weapon, sync_range_indicator, update_weapon_visual,
    player_attack, handle_block, apply_player_delayed_hits, update_player_attack_state,
};
use crate::systems::cursor_not_over_ui;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_player,
                handle_dash_input,
                apply_dash,
                tick_dash_cooldown,
                tick_phase_through,
                apply_knockback,
                animate_player_death,
                toggle_weapon,
                aim_weapon,
                animate_weapon_swing,
                sync_range_indicator,
                update_weapon_visual,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (player_attack, handle_block)
                .run_if(in_state(GameState::Playing))
                .run_if(cursor_not_over_ui),
        )
        .add_systems(
            Update,
            (apply_player_delayed_hits, update_player_attack_state)
                .run_if(in_state(GameState::Playing)),
        );
    }
}
