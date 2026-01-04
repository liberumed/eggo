use bevy::prelude::*;

use crate::resources::GameState;
use crate::systems::{
    aim_weapon, animate_player_death, animate_weapon_swing, apply_dash,
    apply_knockback, apply_player_delayed_hits, cursor_not_over_ui, handle_block,
    handle_dash_input, move_player, player_attack, sync_range_indicator, tick_dash_cooldown,
    tick_phase_through, toggle_weapon, update_weapon_visual,
};

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
            apply_player_delayed_hits.run_if(in_state(GameState::Playing)),
        );
    }
}
