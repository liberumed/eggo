#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropType {
    Pillar,
    Barrel,
    Crate,
    StoneWall,
}

#[derive(Clone)]
pub struct PropDefinition {
    pub prop_type: PropType,
    pub collision_radius: f32,
    pub collision_offset_y: f32,  // Y offset from transform to collision center
    pub base_offset: f32,
    pub destructible: bool,
    pub health: Option<i32>,
}

impl PropDefinition {
    pub fn pillar() -> Self {
        Self {
            prop_type: PropType::Pillar,
            collision_radius: 10.0,
            collision_offset_y: -14.0,  // Visual is +14, collision at base
            base_offset: -28.0,
            destructible: false,
            health: None,
        }
    }

    pub fn barrel() -> Self {
        Self {
            prop_type: PropType::Barrel,
            collision_radius: 8.0,
            collision_offset_y: 0.0,
            base_offset: -14.0,
            destructible: true,
            health: Some(1),
        }
    }

    pub fn crate_box() -> Self {
        Self {
            prop_type: PropType::Crate,
            collision_radius: 9.0,
            collision_offset_y: 0.0,
            base_offset: -9.0,
            destructible: true,
            health: Some(2),
        }
    }

    pub fn stone_wall() -> Self {
        Self {
            prop_type: PropType::StoneWall,
            collision_radius: 14.0,
            collision_offset_y: -10.0,  // Visual is +10, collision at base
            base_offset: -24.0,
            destructible: false,
            health: None,
        }
    }
}

#[derive(Component)]
pub struct Prop {
    pub prop_type: PropType,
}

#[derive(Component)]
pub struct Destructible {
    pub health: i32,
}
