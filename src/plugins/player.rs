use bevy::prelude::*;

use crate::resources::GameState;
use crate::systems::{
    aim_weapon, animate_player, animate_player_death, animate_weapon_swing, apply_knockback,
    handle_block, move_player, player_attack, toggle_weapon,
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
                handle_block,
                player_attack,
                aim_weapon,
                animate_weapon_swing,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
