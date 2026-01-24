use bevy::prelude::*;
use std::f32::consts::TAU;

pub const NUM_DIRECTIONS: usize = 8;

/// Context map for steering decisions
/// Each slot represents interest/danger in a direction (0 = East, rotating counter-clockwise)
#[derive(Clone, Debug)]
pub struct ContextMap {
    pub interest: [f32; NUM_DIRECTIONS],
    pub danger: [f32; NUM_DIRECTIONS],
}

impl Default for ContextMap {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMap {
    pub fn new() -> Self {
        Self {
            interest: [0.0; NUM_DIRECTIONS],
            danger: [0.0; NUM_DIRECTIONS],
        }
    }

    /// Get the unit direction vector for a given slot index
    /// Index 0 = East (1, 0), rotating counter-clockwise
    pub fn direction(index: usize) -> Vec2 {
        let angle = index as f32 * TAU / NUM_DIRECTIONS as f32;
        Vec2::new(angle.cos(), angle.sin())
    }

    /// Clear both maps to zero
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.interest = [0.0; NUM_DIRECTIONS];
        self.danger = [0.0; NUM_DIRECTIONS];
    }

    /// Resolve the context map to find the best movement direction
    /// Returns (direction, strength) where strength is 0-1
    pub fn resolve(&self) -> (Vec2, f32) {
        let mut best_index = 0;
        let mut best_value = f32::NEG_INFINITY;

        for i in 0..NUM_DIRECTIONS {
            let value = self.interest[i] - self.danger[i];
            if value > best_value {
                best_value = value;
                best_index = i;
            }
        }

        let direction = Self::direction(best_index);
        let strength = best_value.max(0.0).min(1.0);

        (direction, strength)
    }
}

/// Cached context map for debug visualization
#[derive(Component, Clone, Debug, Default)]
pub struct ContextMapCache(pub ContextMap);

/// Stores creature's preferred flank angle (radians)
/// Positive = approach from left, Negative = approach from right
#[derive(Component, Clone, Debug)]
pub struct FlankPreference(pub f32);

// ============================================================================
// Behaviors - Functions that fill the context maps
// ============================================================================

/// Add interest toward a target position (e.g., seeking the player)
#[allow(dead_code)]
pub fn seek_interest(map: &mut ContextMap, from_pos: Vec2, target_pos: Vec2) {
    let to_target = (target_pos - from_pos).normalize_or_zero();
    if to_target == Vec2::ZERO {
        return;
    }

    for i in 0..NUM_DIRECTIONS {
        let dir = ContextMap::direction(i);
        // Dot product: 1.0 when perfectly aligned, 0.0 when perpendicular, -1.0 when opposite
        let dot = dir.dot(to_target).max(0.0); // Only positive interest
        map.interest[i] = map.interest[i].max(dot);
    }
}

/// Add danger for obstacles/colliders within look_ahead distance
pub fn obstacle_danger(
    map: &mut ContextMap,
    creature_pos: Vec2,
    colliders: &[(Vec2, Vec2)], // (position, radius as Vec2)
    look_ahead: f32,
) {
    for (obs_pos, obs_radius) in colliders {
        let to_obs = *obs_pos - creature_pos;
        // Use the larger radius for distance calculation
        let effective_radius = obs_radius.x.max(obs_radius.y);
        let dist = to_obs.length() - effective_radius;

        if dist < look_ahead && dist > 0.001 {
            let dir_to_obs = to_obs.normalize();
            let proximity = 1.0 - (dist / look_ahead); // Closer = more danger

            for i in 0..NUM_DIRECTIONS {
                let dir = ContextMap::direction(i);
                let alignment = dir.dot(dir_to_obs).max(0.0);
                let danger = alignment * proximity;
                map.danger[i] = map.danger[i].max(danger);
            }
        }
    }
}

/// Add danger for nearby creatures (separation behavior)
pub fn separation_danger(
    map: &mut ContextMap,
    creature_pos: Vec2,
    other_creatures: &[Vec2],
    separation_radius: f32,
) {
    for other_pos in other_creatures {
        let to_other = *other_pos - creature_pos;
        let dist = to_other.length();

        if dist < separation_radius && dist > 0.001 {
            let dir_to_other = to_other.normalize();
            let proximity = 1.0 - (dist / separation_radius);

            for i in 0..NUM_DIRECTIONS {
                let dir = ContextMap::direction(i);
                let alignment = dir.dot(dir_to_other).max(0.0);
                let danger = alignment * proximity;
                map.danger[i] = map.danger[i].max(danger);
            }
        }
    }
}

/// Add interest toward a rotated direction (for flanking)
/// flank_angle: rotation in radians (positive = counter-clockwise)
/// This causes creatures to approach from an angle rather than directly
pub fn seek_with_flank(
    map: &mut ContextMap,
    creature_pos: Vec2,
    player_pos: Vec2,
    flank_angle: f32,
) {
    let to_player = (player_pos - creature_pos).normalize_or_zero();
    if to_player == Vec2::ZERO {
        return;
    }

    // Rotate the target direction by flank_angle
    let cos_a = flank_angle.cos();
    let sin_a = flank_angle.sin();
    let rotated = Vec2::new(
        to_player.x * cos_a - to_player.y * sin_a,
        to_player.x * sin_a + to_player.y * cos_a,
    );

    for i in 0..NUM_DIRECTIONS {
        let dir = ContextMap::direction(i);
        let dot = dir.dot(rotated).max(0.0);
        map.interest[i] = map.interest[i].max(dot);
    }
}

/// Add danger for directions where other creatures are already positioned relative to player
/// This forces creatures to spread around the player instead of bunching up
pub fn occupied_angle_danger(
    map: &mut ContextMap,
    creature_pos: Vec2,
    player_pos: Vec2,
    other_creatures: &[Vec2],
    angle_spread: f32, // How wide the "occupied" zone is (radians)
) {
    let to_player = player_pos - creature_pos;
    let dist_to_player = to_player.length();
    if dist_to_player < 1.0 {
        return;
    }

    for other_pos in other_creatures {
        let other_to_player = player_pos - *other_pos;
        let other_dist = other_to_player.length();

        // Only consider creatures that are closer to the player than us
        if other_dist >= dist_to_player || other_dist < 1.0 {
            continue;
        }

        // Get the angle this other creature occupies relative to player
        let other_angle = (-other_to_player).y.atan2((-other_to_player).x);

        // Add danger in directions that would put us at the same angle
        for i in 0..NUM_DIRECTIONS {
            let dir = ContextMap::direction(i);
            // Where would we end up if we moved this direction?
            let approach_angle = (-to_player + dir * 10.0).y.atan2((-to_player + dir * 10.0).x);

            // How close is this to the occupied angle?
            let angle_diff = (approach_angle - other_angle).abs();
            let angle_diff = if angle_diff > std::f32::consts::PI {
                std::f32::consts::TAU - angle_diff
            } else {
                angle_diff
            };

            if angle_diff < angle_spread {
                let danger = 1.0 - (angle_diff / angle_spread);
                map.danger[i] = map.danger[i].max(danger * 0.7);
            }
        }
    }
}

/// Add danger for getting too close to the player (maintain attack distance)
pub fn player_proximity_danger(
    map: &mut ContextMap,
    creature_pos: Vec2,
    player_pos: Vec2,
    min_distance: f32,
) {
    let to_player = player_pos - creature_pos;
    let dist = to_player.length();

    if dist < min_distance && dist > 0.001 {
        let dir_to_player = to_player.normalize();
        let proximity = 1.0 - (dist / min_distance);

        for i in 0..NUM_DIRECTIONS {
            let dir = ContextMap::direction(i);
            let alignment = dir.dot(dir_to_player).max(0.0);
            let danger = alignment * proximity * 0.5; // Lower weight than obstacles
            map.danger[i] = map.danger[i].max(danger);
        }
    }
}

/// Add danger for pit hazards
/// pits: slice of (center, edge_radius) tuples
pub fn pit_danger(
    map: &mut ContextMap,
    creature_pos: Vec2,
    pits: &[(Vec2, f32)],
    look_ahead: f32,
) {
    for (pit_center, edge_radius) in pits {
        let to_pit = *pit_center - creature_pos;
        let dist = to_pit.length() - edge_radius;

        if dist < look_ahead && dist > 0.001 {
            let dir_to_pit = to_pit.normalize();
            let proximity = 1.0 - (dist / look_ahead);

            for i in 0..NUM_DIRECTIONS {
                let dir = ContextMap::direction(i);
                let alignment = dir.dot(dir_to_pit).max(0.0);
                let danger = alignment * proximity * 0.6;
                map.danger[i] = map.danger[i].max(danger);
            }
        }
    }
}

/// Add interest for patrol wandering behavior
/// Combines current wander direction with pull back toward origin when near edge
pub fn patrol_interest(
    map: &mut ContextMap,
    creature_pos: Vec2,
    origin: Vec2,
    wander_direction: Vec2,
    patrol_radius: f32,
) {
    let to_origin = origin - creature_pos;
    let dist_from_origin = to_origin.length();

    for i in 0..NUM_DIRECTIONS {
        let dir = ContextMap::direction(i);

        let mut interest = 0.0;

        if wander_direction.length_squared() > 0.01 {
            let wander_dot = dir.dot(wander_direction).max(0.0);
            interest += wander_dot * 0.6;
        }

        if dist_from_origin > patrol_radius * 0.7 && dist_from_origin > 0.001 {
            let to_origin_normalized = to_origin / dist_from_origin;
            let return_dot = dir.dot(to_origin_normalized).max(0.0);
            let pull_strength = (dist_from_origin - patrol_radius * 0.7) / (patrol_radius * 0.3);
            interest += return_dot * pull_strength.min(1.0);
        }

        map.interest[i] = map.interest[i].max(interest);
    }
}

/// Add danger for leaving patrol area (hard boundary)
pub fn patrol_boundary_danger(
    map: &mut ContextMap,
    creature_pos: Vec2,
    origin: Vec2,
    patrol_radius: f32,
) {
    let to_origin = origin - creature_pos;
    let dist_from_origin = to_origin.length();

    if dist_from_origin > patrol_radius * 0.8 && dist_from_origin > 0.001 {
        let away_from_origin = -to_origin / dist_from_origin;
        let boundary_proximity = (dist_from_origin - patrol_radius * 0.8) / (patrol_radius * 0.2);

        for i in 0..NUM_DIRECTIONS {
            let dir = ContextMap::direction(i);
            let alignment = dir.dot(away_from_origin).max(0.0);
            let danger = alignment * boundary_proximity.min(1.0);
            map.danger[i] = map.danger[i].max(danger);
        }
    }
}
