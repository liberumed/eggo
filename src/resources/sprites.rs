use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// Animation data for a single animation
#[derive(Clone, Debug)]
pub struct AnimationData {
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub start_index: usize,
    pub frame_count: usize,
    pub frame_duration_ms: u32,
    pub looping: bool,
}

/// Resource holding the player sprite sheet and animations
#[derive(Resource)]
pub struct PlayerSpriteSheet {
    pub animations: HashMap<String, AnimationData>,
}

// Aseprite JSON structures
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct AsepriteJson {
    pub frames: HashMap<String, AsepriteFrame>,
    pub meta: AsepriteMeta,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct AsepriteFrame {
    pub frame: AsepriteRect,
    pub duration: u32,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct AsepriteRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct AsepriteMeta {
    pub size: AsepriteSize,
    #[serde(rename = "frameTags", default)]
    pub frame_tags: Vec<AsepriteFrameTag>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct AsepriteSize {
    pub w: u32,
    pub h: u32,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct AsepriteFrameTag {
    pub name: String,
    pub from: usize,
    pub to: usize,
}

/// Load a single animation from Aseprite export
fn load_animation(
    name: &str,
    png_path: &'static str,
    json_path: &str,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    looping: bool,
) -> AnimationData {
    let texture: Handle<Image> = asset_server.load(png_path);

    let json_str = std::fs::read_to_string(format!("assets/{}", json_path))
        .expect(&format!("Failed to read {} JSON", name));
    let aseprite: AsepriteJson = serde_json::from_str(&json_str)
        .expect(&format!("Failed to parse {} JSON", name));

    let mut frames: Vec<_> = aseprite.frames.iter().collect();
    frames.sort_by(|a, b| a.0.cmp(b.0));

    let frame_width = frames.first().map(|(_, f)| f.frame.w).unwrap_or(64);
    let frame_height = frames.first().map(|(_, f)| f.frame.h).unwrap_or(64);

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(frame_width, frame_height),
        frames.len() as u32,
        1,
        None,
        None,
    );
    let atlas_layout = texture_atlas_layouts.add(layout);

    let duration = frames.first().map(|(_, f)| f.duration).unwrap_or(100);

    AnimationData {
        texture,
        atlas_layout,
        start_index: 0,
        frame_count: frames.len(),
        frame_duration_ms: duration,
        looping,
    }
}

/// Load and parse Aseprite JSON files, build TextureAtlasLayouts
pub fn load_player_sprite_sheet(
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> PlayerSpriteSheet {
    let mut animations = HashMap::new();

    // Load idle animation
    animations.insert("idle".to_string(), load_animation(
        "idle",
        "sprites/player/player_idle.png",
        "sprites/player/player_idle.json",
        asset_server,
        texture_atlas_layouts,
        true,
    ));

    // Load walk animation
    animations.insert("walk".to_string(), load_animation(
        "walk",
        "sprites/player/player_walk.png",
        "sprites/player/player_walk.json",
        asset_server,
        texture_atlas_layouts,
        true,
    ));

    // Load walk_up animation
    animations.insert("walk_up".to_string(), load_animation(
        "walk_up",
        "sprites/player/player_walk_up.png",
        "sprites/player/player_walk_up.json",
        asset_server,
        texture_atlas_layouts,
        true,
    ));

    // Load walk_down animation
    animations.insert("walk_down".to_string(), load_animation(
        "walk_down",
        "sprites/player/player_walk_down.png",
        "sprites/player/player_walk_down.json",
        asset_server,
        texture_atlas_layouts,
        true,
    ));

    // Load attack animation (non-looping)
    animations.insert("attack".to_string(), load_animation(
        "attack",
        "sprites/player/player_attack.png",
        "sprites/player/player_attack.json",
        asset_server,
        texture_atlas_layouts,
        false,
    ));

    // Load idle with stick animation
    animations.insert("idle_stick".to_string(), load_animation(
        "idle_stick",
        "sprites/player/player_idle_stick.png",
        "sprites/player/player_idle_stick.json",
        asset_server,
        texture_atlas_layouts,
        true,
    ));

    PlayerSpriteSheet { animations }
}
