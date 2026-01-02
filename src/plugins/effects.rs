use bevy::prelude::*;

use crate::systems::{
    animate_blood, animate_dust, animate_magnetized_balls, animate_resource_balls,
    spawn_sprint_dust, tick_hitstop, tick_screen_shake,
};

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animate_blood,
                animate_dust,
                animate_resource_balls,
                animate_magnetized_balls,
                spawn_sprint_dust,
                tick_hitstop,
                tick_screen_shake,
            ),
        );
    }
}
