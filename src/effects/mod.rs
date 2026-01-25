pub mod components;
pub mod game_feel;
pub mod systems;

pub use components::*;
pub use game_feel::*;
pub use systems::*;

use bevy::prelude::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animate_blood,
                animate_dust,
                animate_damage_numbers,
                animate_resource_balls,
                animate_magnetized_balls,
                animate_hit_highlight,
                animate_sprite_hit_highlight,
                spawn_sprint_dust,
                tick_hitstop,
                tick_screen_shake,
            ),
        );
    }
}
