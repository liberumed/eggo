use bevy::prelude::*;

use crate::systems::{camera_follow, update_despawn_timer, update_stun};

pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_follow, update_stun, update_despawn_timer));
    }
}
