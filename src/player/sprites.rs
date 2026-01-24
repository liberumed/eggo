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

// Aseprite JSON structures for json-array format
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct AsepriteJsonArray {
    pub frames: Vec<AsepriteFrameEntry>,
    pub meta: AsepriteMeta,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct AsepriteFrameEntry {
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

/// Non-looping animation tag prefixes
const NON_LOOPING_PREFIXES: &[&str] = &["att_", "hurt_"];

fn is_looping_animation(name: &str) -> bool {
    !NON_LOOPING_PREFIXES.iter().any(|prefix| name.starts_with(prefix))
}

/// Load all tagged animations from an Aseprite export
fn load_tagged_animations(
    png_path: &'static str,
    json_path: &str,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> Vec<(String, AnimationData)> {
    let texture: Handle<Image> = asset_server.load(png_path);

    let json_str = std::fs::read_to_string(format!("assets/{}", json_path))
        .unwrap_or_else(|_| panic!("Failed to read JSON: {}", json_path));
    let aseprite: AsepriteJsonArray = serde_json::from_str(&json_str)
        .unwrap_or_else(|_| panic!("Failed to parse JSON: {}", json_path));

    // Get frame dimensions from the first frame
    let frame_width = aseprite.frames.first().map(|f| f.frame.w).unwrap_or(128);
    let frame_height = aseprite.frames.first().map(|f| f.frame.h).unwrap_or(128);
    let total_frames = aseprite.frames.len() as u32;

    // Create a single atlas layout for the entire spritesheet
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(frame_width, frame_height),
        total_frames,
        1,
        None,
        None,
    );
    let atlas_layout = texture_atlas_layouts.add(layout);

    // Create animation data for each frame tag
    aseprite.meta.frame_tags.iter().map(|tag| {
        let frame_count = tag.to - tag.from + 1;
        let duration = aseprite.frames.get(tag.from)
            .map(|f| f.duration)
            .unwrap_or(100);

        (
            tag.name.clone(),
            AnimationData {
                texture: texture.clone(),
                atlas_layout: atlas_layout.clone(),
                start_index: tag.from,
                frame_count,
                frame_duration_ms: duration,
                looping: is_looping_animation(&tag.name),
            }
        )
    }).collect()
}

/// Load and parse Aseprite JSON files, build TextureAtlasLayouts
pub fn load_player_sprite_sheet(
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> PlayerSpriteSheet {
    let mut animations = HashMap::new();

    // Load all animations from the new tagged spritesheet
    for (name, data) in load_tagged_animations(
        "sprites/player/player_new.png",
        "sprites/player/player_new.json",
        asset_server,
        texture_atlas_layouts,
    ) {
        animations.insert(name, data);
    }

    PlayerSpriteSheet { animations }
}
