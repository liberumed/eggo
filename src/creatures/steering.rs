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

// ============================================================================
// Behaviors - Functions that fill the context maps
// ============================================================================

/// Add interest toward a target position (e.g., seeking the player)
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
