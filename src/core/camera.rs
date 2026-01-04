use bevy::prelude::*;

use crate::components::Player;
use crate::effects::ScreenShake;

pub fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    screen_shake: Res<ScreenShake>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let Ok(mut camera_transform) = camera_query.single_mut() else { return };

    let shake_offset = screen_shake.get_offset();

    camera_transform.translation.x = player_transform.translation.x + shake_offset.x;
    camera_transform.translation.y = player_transform.translation.y + shake_offset.y;
}
