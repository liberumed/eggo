use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Stats {
    pub philosophy: u32,
    pub nature_study: u32,
    pub wisdom: u32,
}
