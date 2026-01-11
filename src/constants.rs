#![allow(dead_code)]

// Z-Layers (rendering order)
// Fixed layers (not Y-sorted)
pub const Z_BACKGROUND: f32 = -10.0;
pub const Z_DEAD: f32 = -8.0;
pub const Z_TARGET_OUTLINE: f32 = -3.0;

// Y-sorted entities use Z range: -1.0 to 1.0 (calculated dynamically)

// Child relative offsets (relative to parent Z)
pub const Z_SHADOW_OFFSET: f32 = -2.0;
pub const Z_CHARACTER_DETAIL: f32 = 0.05;
pub const Z_WEAPON: f32 = 0.1;
pub const Z_PARTICLE: f32 = 0.15;

// Fixed layers above Y-sorted
pub const Z_BLOOD: f32 = 3.0;
pub const Z_UI_WORLD: f32 = 5.0;

// Display
pub const PIXEL_SCALE: f32 = 4.0;

// Camera
pub const CAMERA_BASE_SCALE: f32 = 1.0 / PIXEL_SCALE;  // Default zoom level
pub const CAMERA_ZOOM_OUT_SCALE: f32 = 1.15 / PIXEL_SCALE;  // Zoomed out when moving
pub const CAMERA_ZOOM_SPEED: f32 = 3.0;  // How fast to lerp between zoom levels

// Movement
pub const PLAYER_SPEED: f32 = 100.0;
pub const PLAYER_ACCELERATION: f32 = 2000.0;  // Near-instant response
pub const PLAYER_FRICTION: f32 = 2000.0;      // Near-instant stop
pub const HOSTILE_SPEED: f32 = 55.0;
pub const PROVOKED_SPEED: f32 = 40.0;
pub const KNOCKBACK_FORCE: f32 = 250.0;

// Dash
pub const DASH_SPEED: f32 = 350.0;
pub const DASH_DURATION: f32 = 0.18;
pub const DASH_COOLDOWN: f32 = 0.5;

// Sprint
pub const SPRINT_MIN_MULTIPLIER: f32 = 1.3;   // Starting sprint speed
pub const SPRINT_MAX_MULTIPLIER: f32 = 2.0;   // Max sprint speed after ramp
pub const SPRINT_RAMP_TIME: f32 = 0.8;        // Time to reach max speed
pub const SPRINT_MOMENTUM_FRICTION: f32 = 300.0; // Lower friction when slowing from sprint

// Game Feel
pub const HITSTOP_DURATION: f32 = 0.15;       // 150ms freeze on hit (anime-style impact)
pub const SCREEN_SHAKE_INTENSITY: f32 = 5.0;  // Pixels
pub const SCREEN_SHAKE_DURATION: f32 = 0.15;
pub const HIT_HIGHLIGHT_DURATION: f32 = 0.15; // Red flash duration

// Combat - Attack Timing
pub const ATTACK_HIT_DELAY_PERCENT: f32 = 0.5;  // Hit at 50% into swing animation
pub const ATTACK_COOLDOWN_DURATION: f32 = 1.5;  // Creature wait time between attacks

// Combat - Attack Geometry
pub const WEAPON_OFFSET: (f32, f32) = (-4.0, 6.5);  // Weapon position relative to player
pub const ATTACK_CENTER_OFFSET_Y: f32 = 6.5;        // Vertical offset to center attacks on body
pub const CARDINAL_RIGHT: f32 = 0.0;
pub const CARDINAL_UP: f32 = std::f32::consts::FRAC_PI_2;
pub const CARDINAL_LEFT: f32 = std::f32::consts::PI;
pub const CARDINAL_DOWN: f32 = -std::f32::consts::FRAC_PI_2;

// Combat - Weapon Ranges
pub const COLLISION_RADIUS: f32 = 14.0;
pub const FIST_RANGE: f32 = COLLISION_RADIUS * 1.8;
pub const KNIFE_RANGE: f32 = COLLISION_RADIUS * 4.0;

// Combat - Blocking
pub const BLOCK_KNOCKBACK: f32 = 120.0;
pub const BLOCK_FACING_OFFSET: f32 = 0.4;   // Radians offset for block facing direction
pub const BLOCK_ANGLE_THRESHOLD: f32 = 0.5; // Dot product threshold for valid block

// Combat - Physics
pub const PUSH_RADIUS: f32 = COLLISION_RADIUS * 2.2;  // Larger than all collision checks
pub const PUSH_STRENGTH: f32 = 100.0;                 // Push force multiplier
pub const HOSTILE_SIGHT_RANGE: f32 = 200.0;

// Combat - Visual Indicators
pub const ARC_THICKNESS: f32 = 0.4;   // Thickness of range arc indicators
pub const ARC_SEGMENTS: u32 = 16;     // Smoothness of arc mesh

// Context Steering
pub const OBSTACLE_LOOK_AHEAD: f32 = 50.0;            // How far to check for obstacles
pub const SEPARATION_RADIUS: f32 = 35.0;              // Keep this distance from other creatures
pub const PLAYER_MIN_DISTANCE: f32 = COLLISION_RADIUS * 1.8;  // Don't get too close to player

// Animation Durations
pub const KNIFE_SWING_DURATION: f32 = 0.4;
pub const FIST_SWING_DURATION: f32 = 0.3;
pub const STUN_DURATION: f32 = 1.0;
pub const DEATH_EXPAND_DURATION: f32 = 0.4;
pub const DEATH_COLLAPSE_DURATION: f32 = 0.3;
pub const CORPSE_LIFETIME: f32 = 3.0;
pub const PARTICLE_LIFETIME: f32 = 3.0;

// World Generation
pub const WORLD_SIZE: i32 = 20;
pub const GRID_SPACING: f32 = 25.0;
pub const CREATURE_SPAWN_CHANCE: f64 = 0.6;
pub const HOSTILE_CHANCE: f64 = 0.15;
pub const GLOWING_CHANCE: f64 = 0.3;
