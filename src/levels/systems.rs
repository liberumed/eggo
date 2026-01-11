use bevy::prelude::*;

use crate::core::{Dead, WalkCollider};
use super::CurrentLevel;

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
