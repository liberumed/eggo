use bevy::prelude::*;

use crate::systems::{
    animate_blood, animate_dust, animate_hit_highlight, animate_magnetized_balls,
    animate_resource_balls, animate_sprite_hit_highlight, spawn_sprint_dust, tick_hitstop,
    tick_screen_shake,
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
                animate_hit_highlight,
                animate_sprite_hit_highlight,
                spawn_sprint_dust,
                tick_hitstop,
                tick_screen_shake,
            ),
        );
    }
}
