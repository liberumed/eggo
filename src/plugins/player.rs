use bevy::prelude::*;

use crate::resources::GameState;
use crate::systems::{
    aim_weapon, animate_player, animate_player_death, animate_weapon_swing, apply_knockback,
    cursor_not_over_ui, handle_block, move_player, player_attack, toggle_weapon,
    update_weapon_visual,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_player,
                apply_knockback,
                animate_player,
                animate_player_death,
                toggle_weapon,
                aim_weapon,
                animate_weapon_swing,
                update_weapon_visual,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (player_attack, handle_block)
                .run_if(in_state(GameState::Playing))
                .run_if(cursor_not_over_ui),
        );
    }
}
