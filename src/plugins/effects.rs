use bevy::prelude::*;

use crate::systems::{animate_blood, animate_magnetized_balls, animate_resource_balls};

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate_blood, animate_resource_balls, animate_magnetized_balls),
        );
    }
}
