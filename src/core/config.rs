use bevy::prelude::*;
use configparser::ini::Ini;
use std::path::Path;

use crate::constants::*;

/// Runtime game configuration loaded from INI file with fallback to compiled defaults
#[derive(Resource, Clone)]
pub struct GameConfig {
    // === PLAYER CONFIG ===
    // Movement
    pub player_speed: f32,
    pub player_acceleration: f32,
    pub player_friction: f32,
    // Dash
    pub dash_speed: f32,
    pub dash_duration: f32,
    pub dash_cooldown: f32,
    // Sprint
    pub sprint_min_multiplier: f32,
    pub sprint_max_multiplier: f32,
    pub sprint_ramp_time: f32,
    pub sprint_momentum_friction: f32,
    pub sprint_decel_threshold: f32,
    // Blocking
    pub blocking_speed_multiplier: f32,
    pub block_knockback: f32,
    pub block_facing_offset: f32,
    pub block_angle_threshold: f32,

    // === CREATURE CONFIG ===
    pub hostile_speed: f32,
    pub goblin_sight_range: f32,
    pub attack_cooldown_duration: f32,

    // === SHARED COMBAT CONFIG ===
    pub knockback_force: f32,
    pub attack_hit_delay_percent: f32,
    pub attack_center_offset_y: f32,
    pub collision_radius: f32,
    pub push_strength: f32,
    pub stun_duration: f32,

    // === GAME FEEL ===
    pub hitstop_duration: f32,
    pub screen_shake_intensity: f32,
    pub screen_shake_duration: f32,
    pub hit_highlight_duration: f32,

    // === WEAPON RANGES ===
    pub fist_range: f32,
    pub knife_range: f32,
    pub club_range: f32,
    pub stick_range: f32,
    pub sword_range: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            // Player Movement
            player_speed: PLAYER_SPEED,
            player_acceleration: PLAYER_ACCELERATION,
            player_friction: PLAYER_FRICTION,
            // Player Dash
            dash_speed: DASH_SPEED,
            dash_duration: DASH_DURATION,
            dash_cooldown: DASH_COOLDOWN,
            // Player Sprint
            sprint_min_multiplier: SPRINT_MIN_MULTIPLIER,
            sprint_max_multiplier: SPRINT_MAX_MULTIPLIER,
            sprint_ramp_time: SPRINT_RAMP_TIME,
            sprint_momentum_friction: SPRINT_MOMENTUM_FRICTION,
            sprint_decel_threshold: 1.1,
            // Player Blocking
            blocking_speed_multiplier: 0.4,
            block_knockback: BLOCK_KNOCKBACK,
            block_facing_offset: BLOCK_FACING_OFFSET,
            block_angle_threshold: BLOCK_ANGLE_THRESHOLD,
            // Creature
            hostile_speed: HOSTILE_SPEED,
            goblin_sight_range: GOBLIN_SIGHT_RANGE,
            attack_cooldown_duration: ATTACK_COOLDOWN_DURATION,
            // Shared Combat
            knockback_force: KNOCKBACK_FORCE,
            attack_hit_delay_percent: ATTACK_HIT_DELAY_PERCENT,
            attack_center_offset_y: ATTACK_CENTER_OFFSET_Y,
            collision_radius: COLLISION_RADIUS,
            push_strength: PUSH_STRENGTH,
            stun_duration: STUN_DURATION,
            // Game Feel
            hitstop_duration: HITSTOP_DURATION,
            screen_shake_intensity: SCREEN_SHAKE_INTENSITY,
            screen_shake_duration: SCREEN_SHAKE_DURATION,
            hit_highlight_duration: HIT_HIGHLIGHT_DURATION,
            // Weapon Ranges
            fist_range: FIST_RANGE,
            knife_range: KNIFE_RANGE,
            club_range: CLUB_RANGE,
            stick_range: STICK_RANGE,
            sword_range: SWORD_RANGE,
        }
    }
}

impl GameConfig {
    /// Load configuration from INI file with fallback to defaults for missing entries
    pub fn load_from_file(path: &str) -> Self {
        let mut config = Self::default();

        if !Path::new(path).exists() {
            info!("Config file '{}' not found, using defaults", path);
            return config;
        }

        let mut ini = Ini::new();
        if let Err(e) = ini.load(path) {
            warn!("Failed to parse config file '{}': {}, using defaults", path, e);
            return config;
        }

        info!("Loading config from '{}'", path);

        // Helper macro to reduce boilerplate
        macro_rules! load_float {
            ($section:expr, $key:expr, $field:ident) => {
                if let Some(val) = ini.getfloat($section, $key).ok().flatten() {
                    config.$field = val as f32;
                }
            };
        }

        // Player Movement
        load_float!("player_movement", "speed", player_speed);
        load_float!("player_movement", "acceleration", player_acceleration);
        load_float!("player_movement", "friction", player_friction);

        // Player Dash
        load_float!("player_dash", "speed", dash_speed);
        load_float!("player_dash", "duration", dash_duration);
        load_float!("player_dash", "cooldown", dash_cooldown);

        // Player Sprint
        load_float!("player_sprint", "min_multiplier", sprint_min_multiplier);
        load_float!("player_sprint", "max_multiplier", sprint_max_multiplier);
        load_float!("player_sprint", "ramp_time", sprint_ramp_time);
        load_float!("player_sprint", "momentum_friction", sprint_momentum_friction);
        load_float!("player_sprint", "decel_threshold", sprint_decel_threshold);

        // Player Blocking
        load_float!("player_blocking", "speed_multiplier", blocking_speed_multiplier);
        load_float!("player_blocking", "knockback", block_knockback);
        load_float!("player_blocking", "facing_offset", block_facing_offset);
        load_float!("player_blocking", "angle_threshold", block_angle_threshold);

        // Creature
        load_float!("creature", "hostile_speed", hostile_speed);
        load_float!("creature", "goblin_sight_range", goblin_sight_range);
        load_float!("creature", "attack_cooldown", attack_cooldown_duration);

        // Shared Combat
        load_float!("combat", "knockback_force", knockback_force);
        load_float!("combat", "attack_hit_delay_percent", attack_hit_delay_percent);
        load_float!("combat", "attack_center_offset_y", attack_center_offset_y);
        load_float!("combat", "collision_radius", collision_radius);
        load_float!("combat", "push_strength", push_strength);
        load_float!("combat", "stun_duration", stun_duration);

        // Game Feel
        load_float!("game_feel", "hitstop_duration", hitstop_duration);
        load_float!("game_feel", "screen_shake_intensity", screen_shake_intensity);
        load_float!("game_feel", "screen_shake_duration", screen_shake_duration);
        load_float!("game_feel", "hit_highlight_duration", hit_highlight_duration);

        // Weapon Ranges
        load_float!("weapon", "fist_range", fist_range);
        load_float!("weapon", "knife_range", knife_range);
        load_float!("weapon", "club_range", club_range);
        load_float!("weapon", "stick_range", stick_range);
        load_float!("weapon", "sword_range", sword_range);

        config
    }
}
