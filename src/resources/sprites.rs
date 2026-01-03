use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// Animation data for a single animation
#[derive(Clone, Debug)]
pub struct AnimationData {
    pub start_index: usize,
    pub frame_count: usize,
    pub frame_duration_ms: u32,
    pub looping: bool,
}

/// Resource holding the player sprite sheet and animations
#[derive(Resource)]
pub struct PlayerSpriteSheet {
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
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
pub struct AsepriteFrameTag {
    pub name: String,
    pub from: usize,
    pub to: usize,
}

/// Load and parse Aseprite JSON, build TextureAtlasLayout
pub fn load_player_sprite_sheet(
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> PlayerSpriteSheet {
    // Load texture
    let texture: Handle<Image> = asset_server.load("sprites/player/player_idle.png");

    // Parse JSON
    let json_str = std::fs::read_to_string("assets/sprites/player/player_idle.json")
        .expect("Failed to read player sprite JSON");
    let aseprite: AsepriteJson = serde_json::from_str(&json_str)
        .expect("Failed to parse player sprite JSON");

    // Sort frames by name to ensure correct order
    let mut frames: Vec<_> = aseprite.frames.iter().collect();
    frames.sort_by(|a, b| a.0.cmp(b.0));

    // Build TextureAtlasLayout
    let frame_width = frames.first().map(|(_, f)| f.frame.w).unwrap_or(320);
    let frame_height = frames.first().map(|(_, f)| f.frame.h).unwrap_or(320);

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(frame_width, frame_height),
        frames.len() as u32,
        1,
        None,
        None,
    );
    let atlas_layout = texture_atlas_layouts.add(layout);

    // Build animations
    let mut animations = HashMap::new();

    // If we have frame tags, use them
    if !aseprite.meta.frame_tags.is_empty() {
        for tag in &aseprite.meta.frame_tags {
            animations.insert(tag.name.clone(), AnimationData {
                start_index: tag.from,
                frame_count: tag.to - tag.from + 1,
                frame_duration_ms: frames.get(tag.from).map(|(_, f)| f.duration).unwrap_or(100),
                looping: true,
            });
        }
    } else {
        // No tags - treat entire sheet as one "idle" animation
        let duration = frames.first().map(|(_, f)| f.duration).unwrap_or(100);
        animations.insert("idle".to_string(), AnimationData {
            start_index: 0,
            frame_count: frames.len(),
            frame_duration_ms: duration,
            looping: true,
        });
    }

    PlayerSpriteSheet {
        texture,
        atlas_layout,
        animations,
    }
}
