use bevy::prelude::*;

use crate::core::{Dead, GameState, WalkCollider};
use crate::creatures::Hostile;
use crate::player::Player;
use super::{CurrentLevel, WinZone, WinZoneTimer, WinZoneTimerText};

const WIN_ZONE_TIME: f32 = 3.0;

#[derive(Component)]
pub struct BoundToLevel;

pub fn enforce_level_bounds(
    current_level: Res<CurrentLevel>,
    mut query: Query<(&mut Transform, Option<&WalkCollider>), (With<BoundToLevel>, Without<Dead>)>,
) {
    let Some(level) = current_level.data.as_ref() else { return };

    for (mut transform, walk_collider) in &mut query {
        let offset_y = walk_collider.map(|c| c.offset_y).unwrap_or(0.0);

        let feet_pos = Vec2::new(
            transform.translation.x,
            transform.translation.y + offset_y,
        );

        let clamped_feet = level.clamp_to_walkable(feet_pos);

        if clamped_feet != feet_pos {
            transform.translation.x = clamped_feet.x;
            transform.translation.y = clamped_feet.y - offset_y;
        }
    }
}

pub fn check_win_zone(
    time: Res<Time>,
    mut timer: ResMut<WinZoneTimer>,
    mut next_state: ResMut<NextState<GameState>>,
    enemies_query: Query<(), (With<Hostile>, Without<Dead>)>,
    player_query: Query<&Transform, (With<Player>, Without<Dead>)>,
    win_zone_query: Query<(&Transform, &WinZone)>,
    mut timer_text_query: Query<&mut Text2d, With<WinZoneTimerText>>,
) {
    let enemies_alive = !enemies_query.is_empty();

    // Update timer text visibility and content
    if let Ok(mut text) = timer_text_query.single_mut() {
        if enemies_alive {
            **text = "".to_string(); // Don't show text while enemies alive
        } else if timer.0 > 0.0 {
            let remaining = (WIN_ZONE_TIME - timer.0).ceil() as i32;
            **text = format!("{}", remaining.max(1));
        } else {
            **text = "".to_string();
        }
    }

    // Need all enemies dead
    if enemies_alive {
        timer.0 = 0.0;
        return;
    }

    // Need player alive
    let Ok(player_transform) = player_query.single() else {
        timer.0 = 0.0;
        return;
    };

    // Need win zone in level
    let Ok((zone_transform, win_zone)) = win_zone_query.single() else {
        return;
    };

    // Check if player is inside win zone circle
    let player_pos = player_transform.translation.truncate();
    let zone_pos = zone_transform.translation.truncate();
    let distance = player_pos.distance(zone_pos);

    if distance <= win_zone.radius {
        timer.0 += time.delta_secs();
        if timer.0 >= WIN_ZONE_TIME {
            next_state.set(GameState::Victory);
        }
    } else {
        timer.0 = 0.0;
    }
}
