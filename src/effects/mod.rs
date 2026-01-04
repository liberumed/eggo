pub mod components;
pub mod game_feel;
pub mod systems;

pub use components::*;
pub use game_feel::*;

use bevy::prelude::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hitstop>()
            .init_resource::<ScreenShake>()
            .add_systems(
                Update,
                (
                    systems::animate_blood,
                    systems::animate_dust,
                    systems::animate_resource_balls,
                    systems::animate_magnetized_balls,
                    systems::animate_hit_highlight,
                    systems::animate_sprite_hit_highlight,
                    systems::spawn_sprint_dust,
                    systems::tick_hitstop,
                    systems::tick_screen_shake,
                ),
            );
    }
}
