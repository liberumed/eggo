// Z-Layers (rendering order)
pub const Z_BACKGROUND: f32 = -10.0;
pub const Z_SHADOW: f32 = -5.0;
pub const Z_DEAD: f32 = -2.0;
pub const Z_CREATURE: f32 = 0.0;
pub const Z_PLAYER: f32 = 1.0;
pub const Z_CHARACTER_DETAIL: f32 = 0.1;
pub const Z_WEAPON: f32 = 0.2;
pub const Z_PARTICLE: f32 = 0.3;
pub const Z_UI_WORLD: f32 = 5.0;
pub const Z_BLOOD: f32 = 3.0;
pub const Z_TARGET_OUTLINE: f32 = -0.5;

// Display
pub const PIXEL_SCALE: f32 = 4.0;

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

// Game Feel
pub const HITSTOP_DURATION: f32 = 0.06;       // 60ms freeze on hit
pub const SCREEN_SHAKE_INTENSITY: f32 = 3.0;  // Pixels
pub const SCREEN_SHAKE_DURATION: f32 = 0.15;

// Combat
pub const COLLISION_RADIUS: f32 = 14.0;
pub const HOSTILE_SIGHT_RANGE: f32 = 100.0;
pub const BLOCK_KNOCKBACK: f32 = 120.0;
pub const FIST_RANGE: f32 = COLLISION_RADIUS * 1.8;
pub const KNIFE_RANGE: f32 = COLLISION_RADIUS * 4.0;

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
