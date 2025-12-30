use bevy::prelude::*;

use crate::resources::GameState;
use crate::systems::{
    aim_knife, animate_player, animate_weapon_swing, apply_knockback, knife_attack, move_player,
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
                knife_attack,
                aim_knife,
                animate_weapon_swing,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
