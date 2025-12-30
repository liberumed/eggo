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
pub const PLAYER_SPEED: f32 = 75.0;
pub const HOSTILE_SPEED: f32 = 30.0;
pub const KNOCKBACK_FORCE: f32 = 250.0;

// Combat
pub const COLLISION_RADIUS: f32 = 14.0;
pub const HOSTILE_SIGHT_RANGE: f32 = 100.0;
pub const FIST_RANGE: f32 = COLLISION_RADIUS * 1.8;
pub const KNIFE_RANGE: f32 = COLLISION_RADIUS * 3.5;

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
