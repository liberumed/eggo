use bevy::prelude::*;

use crate::components::*;

pub fn update_stun(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Stunned)>,
) {
    for (entity, mut stunned) in &mut query {
        stunned.0 -= time.delta_secs();
        if stunned.0 <= 0.0 {
            commands.entity(entity).remove::<Stunned>();
        }
    }
}

pub fn update_despawn_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DespawnTimer), With<Dead>>,
) {
    let dt = time.delta_secs();
    for (entity, mut timer) in &mut query {
        timer.0 -= dt;
        if timer.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
