use bevy::prelude::*;

use crate::core::{CharacterAssets, Dead, DeathAnimation, GameConfig, GameState, Knockback, WalkCollider};
use crate::creatures::Hostile;
use crate::player::{Player, PlayerSpriteSheet};
use super::{CurrentLevel, Pit, WinZone, WinZoneTimer, WinZoneTimerText};

const WIN_ZONE_TIME: f32 = 5.0;
const WAVE_TRIGGER_TIME: f32 = 2.0;
const WAVE_SPAWN_INTERVAL: f32 = 3.0;
const WAVE_GOBLINS_PER_SIDE: u32 = 4;

#[derive(Resource, Default)]
pub struct WaveSpawnState {
    pub triggered: bool,
    pub spawn_timer: f32,
    pub spawned_left: u32,
    pub spawned_right: u32,
}
const PIT_EDGE_RESISTANCE: f32 = 80.0;
const PIT_FALL_DURATION: f32 = 0.5;

#[derive(Component)]
pub struct BoundToLevel;

#[derive(Component)]
pub struct FallingIntoPit {
    pub timer: f32,
    pub pit_center: Vec2,
}

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
    mut wave_state: ResMut<WaveSpawnState>,
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
            **text = "".to_string();
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

        // Trigger wave at WAVE_TRIGGER_TIME if not yet triggered
        if timer.0 >= WAVE_TRIGGER_TIME && !wave_state.triggered {
            wave_state.triggered = true;
            wave_state.spawn_timer = WAVE_SPAWN_INTERVAL; // Spawn first pair immediately
        }

        if timer.0 >= WIN_ZONE_TIME {
            next_state.set(GameState::Victory);
        }
    } else {
        timer.0 = 0.0;
    }
}

pub fn spawn_wave_goblins(
    mut commands: Commands,
    time: Res<Time>,
    mut wave_state: ResMut<WaveSpawnState>,
    config: Res<GameConfig>,
    character_assets: Res<CharacterAssets>,
    player_sprite_sheet: Res<PlayerSpriteSheet>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !wave_state.triggered {
        return;
    }

    let total_spawned = wave_state.spawned_left + wave_state.spawned_right;
    let max_spawns = WAVE_GOBLINS_PER_SIDE * 2;

    if total_spawned >= max_spawns {
        return;
    }

    wave_state.spawn_timer += time.delta_secs();

    if wave_state.spawn_timer >= WAVE_SPAWN_INTERVAL {
        wave_state.spawn_timer = 0.0;

        let y_positions = [920.0, 940.0, 960.0, 980.0];
        let pair_index = wave_state.spawned_left as usize;
        let spawn_y = y_positions.get(pair_index).copied().unwrap_or(950.0);
        let pentagram_pos = Vec2::new(0.0, 1050.0);

        if wave_state.spawned_left < WAVE_GOBLINS_PER_SIDE {
            let left_pos = Vec2::new(-250.0, spawn_y);
            crate::creatures::spawn_goblin(
                &mut commands,
                &config,
                &character_assets,
                &player_sprite_sheet,
                &mut meshes,
                &mut materials,
                left_pos,
                Some(pentagram_pos),
            );
            wave_state.spawned_left += 1;
        }

        if wave_state.spawned_right < WAVE_GOBLINS_PER_SIDE {
            let right_pos = Vec2::new(250.0, spawn_y);
            crate::creatures::spawn_goblin(
                &mut commands,
                &config,
                &character_assets,
                &player_sprite_sheet,
                &mut meshes,
                &mut materials,
                right_pos,
                Some(pentagram_pos),
            );
            wave_state.spawned_right += 1;
        }
    }
}

pub fn apply_pit_edge_resistance(
    time: Res<Time>,
    pit_query: Query<(&Transform, &Pit)>,
    mut entity_query: Query<(&mut Transform, Option<&WalkCollider>), (With<BoundToLevel>, Without<Dead>, Without<FallingIntoPit>, Without<Pit>, Without<Knockback>)>,
) {
    for (mut entity_transform, walk_collider) in &mut entity_query {
        let offset_y = walk_collider.map(|c| c.offset_y).unwrap_or(0.0);
        let feet_pos = Vec2::new(
            entity_transform.translation.x,
            entity_transform.translation.y + offset_y,
        );

        for (pit_transform, pit) in &pit_query {
            let pit_center = pit_transform.translation.truncate();
            let to_entity = feet_pos - pit_center;
            let distance = to_entity.length();

            if distance >= pit.radius && distance < pit.edge_radius {
                let edge_depth = 1.0 - (distance - pit.radius) / (pit.edge_radius - pit.radius);
                let resistance = edge_depth.powi(2) * PIT_EDGE_RESISTANCE;
                let push_dir = to_entity.normalize_or_zero();
                let push = push_dir * resistance * time.delta_secs();

                entity_transform.translation.x += push.x;
                entity_transform.translation.y += push.y;
            }
        }
    }
}

pub fn check_pit_fall(
    mut commands: Commands,
    pit_query: Query<(&Transform, &Pit)>,
    entity_query: Query<(Entity, &Transform, Option<&WalkCollider>), (With<BoundToLevel>, Without<Dead>, Without<FallingIntoPit>)>,
) {
    for (entity, entity_transform, walk_collider) in &entity_query {
        let offset_y = walk_collider.map(|c| c.offset_y).unwrap_or(0.0);
        let feet_pos = Vec2::new(
            entity_transform.translation.x,
            entity_transform.translation.y + offset_y,
        );

        for (pit_transform, pit) in &pit_query {
            let pit_center = pit_transform.translation.truncate();
            let distance = feet_pos.distance(pit_center);

            if distance < pit.radius {
                commands.entity(entity).insert(FallingIntoPit {
                    timer: 0.0,
                    pit_center,
                });
                break;
            }
        }
    }
}

pub fn animate_pit_fall(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut FallingIntoPit, Option<&WalkCollider>)>,
) {
    for (entity, mut transform, mut falling, walk_collider) in &mut query {
        falling.timer += time.delta_secs();
        let t = (falling.timer / PIT_FALL_DURATION).min(1.0);

        let offset_y = walk_collider.map(|c| c.offset_y).unwrap_or(0.0);
        let current_feet = Vec2::new(
            transform.translation.x,
            transform.translation.y + offset_y,
        );

        let new_feet = current_feet.lerp(falling.pit_center, t * 3.0 * time.delta_secs());
        transform.translation.x = new_feet.x;
        transform.translation.y = new_feet.y - offset_y;

        let scale = (1.0 - t * 2.0).max(0.0);
        transform.scale = Vec3::splat(scale);

        if falling.timer >= PIT_FALL_DURATION {
            commands.entity(entity).insert((Dead, DeathAnimation { timer: 0.0, stage: 2 }));
            commands.entity(entity).remove::<FallingIntoPit>();
        }
    }
}
