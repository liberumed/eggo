use bevy::prelude::*;
use rand::Rng;

use crate::components::{StaticCollider, YSorted};
use crate::constants::{COLLISION_RADIUS, GRID_SPACING, WORLD_SIZE, Z_SHADOW_OFFSET};
use crate::data::{Destructible, Prop, PropDefinition, PropType};

#[derive(Resource)]
pub struct WorldAssets {
    // Pillar
    pub pillar_mesh: Handle<Mesh>,
    pub pillar_material: Handle<ColorMaterial>,
    pub pillar_top_mesh: Handle<Mesh>,
    pub pillar_top_material: Handle<ColorMaterial>,
    pub pillar_shade_mesh: Handle<Mesh>,
    pub pillar_shade_material: Handle<ColorMaterial>,

    // Barrel
    pub barrel_mesh: Handle<Mesh>,
    pub barrel_material: Handle<ColorMaterial>,
    pub barrel_top_mesh: Handle<Mesh>,
    pub barrel_top_material: Handle<ColorMaterial>,
    pub barrel_band_mesh: Handle<Mesh>,
    pub barrel_band_material: Handle<ColorMaterial>,

    // Crate
    pub crate_mesh: Handle<Mesh>,
    pub crate_material: Handle<ColorMaterial>,
    pub crate_cross_mesh: Handle<Mesh>,
    pub crate_cross_material: Handle<ColorMaterial>,

    // Stone wall
    pub wall_mesh: Handle<Mesh>,
    pub wall_material: Handle<ColorMaterial>,
    pub wall_top_mesh: Handle<Mesh>,
    pub wall_top_material: Handle<ColorMaterial>,

    // Shared
    pub shadow_mesh: Handle<Mesh>,
    pub shadow_material: Handle<ColorMaterial>,
}

impl WorldAssets {
    pub fn load(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        Self {
            // Pillar - tall stone column
            pillar_mesh: meshes.add(Rectangle::new(20.0, 56.0)),
            pillar_material: materials.add(Color::srgb(0.5, 0.48, 0.45)),
            pillar_top_mesh: meshes.add(Rectangle::new(20.0, 6.0)),
            pillar_top_material: materials.add(Color::srgb(0.65, 0.62, 0.58)),
            pillar_shade_mesh: meshes.add(Rectangle::new(6.0, 56.0)),
            pillar_shade_material: materials.add(Color::srgba(0.0, 0.0, 0.0, 0.15)),

            // Barrel - wooden barrel
            barrel_mesh: meshes.add(Ellipse::new(10.0, 14.0)),
            barrel_material: materials.add(Color::srgb(0.55, 0.35, 0.2)),
            barrel_top_mesh: meshes.add(Ellipse::new(8.0, 4.0)),
            barrel_top_material: materials.add(Color::srgb(0.65, 0.45, 0.3)),
            barrel_band_mesh: meshes.add(Rectangle::new(22.0, 2.0)),
            barrel_band_material: materials.add(Color::srgb(0.3, 0.3, 0.3)),

            // Crate - wooden box
            crate_mesh: meshes.add(Rectangle::new(18.0, 18.0)),
            crate_material: materials.add(Color::srgb(0.6, 0.45, 0.25)),
            crate_cross_mesh: meshes.add(Rectangle::new(16.0, 2.0)),
            crate_cross_material: materials.add(Color::srgb(0.45, 0.32, 0.18)),

            // Stone wall segment
            wall_mesh: meshes.add(Rectangle::new(32.0, 48.0)),
            wall_material: materials.add(Color::srgb(0.45, 0.43, 0.4)),
            wall_top_mesh: meshes.add(Rectangle::new(32.0, 6.0)),
            wall_top_material: materials.add(Color::srgb(0.55, 0.53, 0.5)),

            // Shared shadow
            shadow_mesh: meshes.add(Ellipse::new(12.0, 6.0)),
            shadow_material: materials.add(Color::srgba(0.0, 0.0, 0.0, 0.3)),
        }
    }
}

pub fn spawn_prop(
    commands: &mut Commands,
    assets: &WorldAssets,
    definition: &PropDefinition,
    position: Vec2,
) {
    let mut entity = commands.spawn((
        Prop { prop_type: definition.prop_type },
        YSorted { base_offset: definition.base_offset },
        StaticCollider {
            radius: definition.collision_radius,
            offset_y: definition.collision_offset_y,
        },
    ));

    if definition.destructible {
        if let Some(health) = definition.health {
            entity.insert(Destructible { health });
        }
    }

    match definition.prop_type {
        PropType::Pillar => spawn_pillar_visual(&mut entity, assets, position),
        PropType::Barrel => spawn_barrel_visual(&mut entity, assets, position),
        PropType::Crate => spawn_crate_visual(&mut entity, assets, position),
        PropType::StoneWall => spawn_wall_visual(&mut entity, assets, position),
    }
}

fn spawn_pillar_visual(entity: &mut EntityCommands, assets: &WorldAssets, position: Vec2) {
    entity.insert((
        Mesh2d(assets.pillar_mesh.clone()),
        MeshMaterial2d(assets.pillar_material.clone()),
        Transform::from_xyz(position.x, position.y + 14.0, 0.0),
    ));

    entity.with_children(|parent| {
        parent.spawn((
            Mesh2d(assets.shadow_mesh.clone()),
            MeshMaterial2d(assets.shadow_material.clone()),
            Transform::from_xyz(1.0, -25.0, Z_SHADOW_OFFSET),
        ));
        parent.spawn((
            Mesh2d(assets.pillar_top_mesh.clone()),
            MeshMaterial2d(assets.pillar_top_material.clone()),
            Transform::from_xyz(0.0, 28.0, 0.001),
        ));
        parent.spawn((
            Mesh2d(assets.pillar_shade_mesh.clone()),
            MeshMaterial2d(assets.pillar_shade_material.clone()),
            Transform::from_xyz(-5.0, 0.0, 0.001),
        ));
    });
}

fn spawn_barrel_visual(entity: &mut EntityCommands, assets: &WorldAssets, position: Vec2) {
    entity.insert((
        Mesh2d(assets.barrel_mesh.clone()),
        MeshMaterial2d(assets.barrel_material.clone()),
        Transform::from_xyz(position.x, position.y, 0.0),
    ));

    entity.with_children(|parent| {
        parent.spawn((
            Mesh2d(assets.shadow_mesh.clone()),
            MeshMaterial2d(assets.shadow_material.clone()),
            Transform::from_xyz(1.0, -11.0, Z_SHADOW_OFFSET),
        ));
        parent.spawn((
            Mesh2d(assets.barrel_top_mesh.clone()),
            MeshMaterial2d(assets.barrel_top_material.clone()),
            Transform::from_xyz(0.0, 10.0, 0.001),
        ));
        // Metal bands
        parent.spawn((
            Mesh2d(assets.barrel_band_mesh.clone()),
            MeshMaterial2d(assets.barrel_band_material.clone()),
            Transform::from_xyz(0.0, 6.0, 0.001),
        ));
        parent.spawn((
            Mesh2d(assets.barrel_band_mesh.clone()),
            MeshMaterial2d(assets.barrel_band_material.clone()),
            Transform::from_xyz(0.0, -6.0, 0.001),
        ));
    });
}

fn spawn_crate_visual(entity: &mut EntityCommands, assets: &WorldAssets, position: Vec2) {
    entity.insert((
        Mesh2d(assets.crate_mesh.clone()),
        MeshMaterial2d(assets.crate_material.clone()),
        Transform::from_xyz(position.x, position.y, 0.0),
    ));

    entity.with_children(|parent| {
        parent.spawn((
            Mesh2d(assets.shadow_mesh.clone()),
            MeshMaterial2d(assets.shadow_material.clone()),
            Transform::from_xyz(1.0, -10.0, Z_SHADOW_OFFSET),
        ));
        // Cross planks
        parent.spawn((
            Mesh2d(assets.crate_cross_mesh.clone()),
            MeshMaterial2d(assets.crate_cross_material.clone()),
            Transform::from_xyz(0.0, 0.0, 0.001).with_rotation(Quat::from_rotation_z(0.785)),
        ));
        parent.spawn((
            Mesh2d(assets.crate_cross_mesh.clone()),
            MeshMaterial2d(assets.crate_cross_material.clone()),
            Transform::from_xyz(0.0, 0.0, 0.002).with_rotation(Quat::from_rotation_z(-0.785)),
        ));
    });
}

fn spawn_wall_visual(entity: &mut EntityCommands, assets: &WorldAssets, position: Vec2) {
    entity.insert((
        Mesh2d(assets.wall_mesh.clone()),
        MeshMaterial2d(assets.wall_material.clone()),
        Transform::from_xyz(position.x, position.y + 10.0, 0.0),
    ));

    entity.with_children(|parent| {
        parent.spawn((
            Mesh2d(assets.shadow_mesh.clone()),
            MeshMaterial2d(assets.shadow_material.clone()),
            Transform::from_xyz(1.0, -21.0, Z_SHADOW_OFFSET),
        ));
        parent.spawn((
            Mesh2d(assets.wall_top_mesh.clone()),
            MeshMaterial2d(assets.wall_top_material.clone()),
            Transform::from_xyz(0.0, 24.0, 0.001),
        ));
    });
}

pub fn spawn_world_props(commands: &mut Commands, assets: &WorldAssets) {
    let mut rng = rand::rng();
    let world_size = WORLD_SIZE as f32 * GRID_SPACING;
    let min_distance = COLLISION_RADIUS * 4.0;
    let mut positions: Vec<Vec2> = Vec::new();

    let prop_count = 20;

    for _ in 0..prop_count * 3 {
        if positions.len() >= prop_count {
            break;
        }

        let x = rng.random_range(-world_size..world_size);
        let y = rng.random_range(-world_size..world_size);

        if x.abs() < COLLISION_RADIUS * 6.0 && y.abs() < COLLISION_RADIUS * 6.0 {
            continue;
        }

        let pos = Vec2::new(x, y);
        let too_close = positions.iter().any(|p| p.distance(pos) < min_distance);
        if too_close {
            continue;
        }

        positions.push(pos);

        // Random prop type
        let definition = match rng.random_range(0..4) {
            0 => PropDefinition::pillar(),
            1 => PropDefinition::barrel(),
            2 => PropDefinition::crate_box(),
            _ => PropDefinition::pillar(),
        };

        spawn_prop(commands, assets, &definition, pos);
    }
}
