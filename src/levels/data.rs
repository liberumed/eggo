use bevy::prelude::*;
use serde::Deserialize;

use crate::inventory::ItemId;

#[derive(Clone, Debug, Deserialize)]
pub struct LevelBounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl LevelBounds {
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    pub fn clamp(&self, point: Vec2) -> Vec2 {
        Vec2::new(
            point.x.clamp(self.min.x, self.max.x),
            point.y.clamp(self.min.y, self.max.y),
        )
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct WalkableRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl WalkableRect {
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    pub fn clamp(&self, point: Vec2) -> Vec2 {
        Vec2::new(
            point.x.clamp(self.min.x, self.max.x),
            point.y.clamp(self.min.y, self.max.y),
        )
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum EntityType {
    Goblin,
    Pillar,
    Barrel,
    Crate,
    Crate2,
    Item { item_id: ItemId, quantity: u32 },
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpawnEntry {
    pub entity_type: EntityType,
    pub position: Vec2,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WinZoneData {
    pub position: Vec2,
    pub radius: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LevelData {
    pub name: String,
    pub bounds: LevelBounds,
    pub walkable: Vec<WalkableRect>,
    pub player_spawn: Vec2,
    pub spawns: Vec<SpawnEntry>,
    #[serde(default)]
    pub win_zone: Option<WinZoneData>,
}

impl LevelData {
    pub fn load_from_file(path: &str) -> Self {
        let contents = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read level file: {}", path));
        ron::from_str(&contents)
            .unwrap_or_else(|e| panic!("Failed to parse level file {}: {}", path, e))
    }

    pub fn is_walkable(&self, point: Vec2) -> bool {
        self.walkable.iter().any(|rect| rect.contains(point))
    }

    pub fn clamp_to_walkable(&self, point: Vec2) -> Vec2 {
        if self.is_walkable(point) {
            return point;
        }

        let mut best_point = point;
        let mut best_dist = f32::MAX;

        for rect in &self.walkable {
            let clamped = rect.clamp(point);
            let dist = (clamped - point).length_squared();
            if dist < best_dist {
                best_dist = dist;
                best_point = clamped;
            }
        }

        best_point
    }
}

#[derive(Resource, Default)]
pub struct CurrentLevel {
    pub data: Option<LevelData>,
}

impl CurrentLevel {
    pub fn load(&mut self, path: &str) {
        self.data = Some(LevelData::load_from_file(path));
    }

    pub fn bounds(&self) -> Option<&LevelBounds> {
        self.data.as_ref().map(|d| &d.bounds)
    }

    pub fn clamp_to_walkable(&self, point: Vec2) -> Vec2 {
        self.data
            .as_ref()
            .map(|d| d.clamp_to_walkable(point))
            .unwrap_or(point)
    }

    pub fn is_walkable(&self, point: Vec2) -> bool {
        self.data.as_ref().map(|d| d.is_walkable(point)).unwrap_or(true)
    }

    pub fn win_zone(&self) -> Option<&WinZoneData> {
        self.data.as_ref().and_then(|d| d.win_zone.as_ref())
    }
}
