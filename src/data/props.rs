#![allow(dead_code)]

use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropType {
    Pillar,
    Barrel,
    Crate,
    StoneWall,
}

/// A single visual layer (mesh + material + transform offset)
#[derive(Clone)]
pub struct PropMeshLayer {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
    pub offset: Vec3,
    pub rotation: f32,
}

/// Visual definition for a prop
#[derive(Clone)]
pub struct PropVisual {
    /// Main body mesh
    pub body: PropMeshLayer,
    /// Shadow mesh (rendered below)
    pub shadow: Option<PropMeshLayer>,
    /// Detail layers (rendered above body)
    pub details: Vec<PropMeshLayer>,
}

/// Complete definition of a prop - single source of truth
#[derive(Clone)]
pub struct PropDefinition {
    pub prop_type: PropType,
    pub collision_radius_x: f32,
    pub collision_radius_y: f32,
    pub collision_offset_y: f32,
    pub base_offset: f32,
    pub visual_offset_y: f32,
    pub destructible: bool,
    pub health: Option<i32>,
    pub visual: PropVisual,
}

/// Registry of all props - loaded at startup
#[derive(Resource)]
pub struct PropRegistry {
    pub props: HashMap<PropType, PropDefinition>,
    pub shadow_mesh: Handle<Mesh>,
    pub shadow_material: Handle<ColorMaterial>,
}

impl PropRegistry {
    pub fn get(&self, prop_type: PropType) -> Option<&PropDefinition> {
        self.props.get(&prop_type)
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

/// Builds the prop registry with all prop definitions
pub fn build_prop_registry(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> PropRegistry {
    let mut props = HashMap::new();

    // Shared shadow
    let shadow_mesh = meshes.add(Ellipse::new(12.0, 6.0));
    let shadow_material = materials.add(Color::srgba(0.0, 0.0, 0.0, 0.3));

    // Pillar (tall stone column)
    props.insert(PropType::Pillar, PropDefinition {
        prop_type: PropType::Pillar,
        collision_radius_x: 10.0,
        collision_radius_y: 6.0,
        collision_offset_y: -53.0,  // Match shadow (ground footprint)
        base_offset: -56.0,
        visual_offset_y: 28.0,
        destructible: false,
        health: None,
        visual: PropVisual {
            body: PropMeshLayer {
                mesh: meshes.add(Rectangle::new(20.0, 112.0)),
                material: materials.add(Color::srgb(0.5, 0.48, 0.45)),
                offset: Vec3::ZERO,
                rotation: 0.0,
            },
            shadow: Some(PropMeshLayer {
                mesh: shadow_mesh.clone(),
                material: shadow_material.clone(),
                offset: Vec3::new(1.0, -53.0, 0.0),
                rotation: 0.0,
            }),
            details: vec![
                PropMeshLayer {
                    mesh: meshes.add(Rectangle::new(20.0, 6.0)),
                    material: materials.add(Color::srgb(0.65, 0.62, 0.58)),
                    offset: Vec3::new(0.0, 56.0, 0.0),
                    rotation: 0.0,
                },
                PropMeshLayer {
                    mesh: meshes.add(Rectangle::new(6.0, 112.0)),
                    material: materials.add(Color::srgba(0.0, 0.0, 0.0, 0.15)),
                    offset: Vec3::new(-5.0, 0.0, 0.0),
                    rotation: 0.0,
                },
            ],
        },
    });

    // Barrel
    props.insert(PropType::Barrel, PropDefinition {
        prop_type: PropType::Barrel,
        collision_radius_x: 10.0,
        collision_radius_y: 6.0,
        collision_offset_y: -11.0,  // Match shadow (ground footprint)
        base_offset: -14.0,
        visual_offset_y: 0.0,
        destructible: true,
        health: Some(1),
        visual: PropVisual {
            body: PropMeshLayer {
                mesh: meshes.add(Ellipse::new(10.0, 14.0)),
                material: materials.add(Color::srgb(0.55, 0.35, 0.2)),
                offset: Vec3::ZERO,
                rotation: 0.0,
            },
            shadow: Some(PropMeshLayer {
                mesh: shadow_mesh.clone(),
                material: shadow_material.clone(),
                offset: Vec3::new(1.0, -11.0, 0.0),
                rotation: 0.0,
            }),
            details: vec![
                PropMeshLayer {
                    mesh: meshes.add(Ellipse::new(8.0, 4.0)),
                    material: materials.add(Color::srgb(0.65, 0.45, 0.3)),
                    offset: Vec3::new(0.0, 10.0, 0.0),
                    rotation: 0.0,
                },
                PropMeshLayer {
                    mesh: meshes.add(Rectangle::new(22.0, 2.0)),
                    material: materials.add(Color::srgb(0.3, 0.3, 0.3)),
                    offset: Vec3::new(0.0, 6.0, 0.0),
                    rotation: 0.0,
                },
                PropMeshLayer {
                    mesh: meshes.add(Rectangle::new(22.0, 2.0)),
                    material: materials.add(Color::srgb(0.3, 0.3, 0.3)),
                    offset: Vec3::new(0.0, -6.0, 0.0),
                    rotation: 0.0,
                },
            ],
        },
    });

    // Crate
    let crate_cross_mesh = meshes.add(Rectangle::new(16.0, 2.0));
    let crate_cross_material = materials.add(Color::srgb(0.45, 0.32, 0.18));
    props.insert(PropType::Crate, PropDefinition {
        prop_type: PropType::Crate,
        collision_radius_x: 9.0,
        collision_radius_y: 5.0,
        collision_offset_y: -10.0,  // Match shadow (ground footprint)
        base_offset: -9.0,
        visual_offset_y: 0.0,
        destructible: true,
        health: Some(2),
        visual: PropVisual {
            body: PropMeshLayer {
                mesh: meshes.add(Rectangle::new(18.0, 18.0)),
                material: materials.add(Color::srgb(0.6, 0.45, 0.25)),
                offset: Vec3::ZERO,
                rotation: 0.0,
            },
            shadow: Some(PropMeshLayer {
                mesh: shadow_mesh.clone(),
                material: shadow_material.clone(),
                offset: Vec3::new(1.0, -10.0, 0.0),
                rotation: 0.0,
            }),
            details: vec![
                PropMeshLayer {
                    mesh: crate_cross_mesh.clone(),
                    material: crate_cross_material.clone(),
                    offset: Vec3::ZERO,
                    rotation: 0.785,
                },
                PropMeshLayer {
                    mesh: crate_cross_mesh.clone(),
                    material: crate_cross_material.clone(),
                    offset: Vec3::ZERO,
                    rotation: -0.785,
                },
            ],
        },
    });

    // Stone Wall
    props.insert(PropType::StoneWall, PropDefinition {
        prop_type: PropType::StoneWall,
        collision_radius_x: 16.0,
        collision_radius_y: 8.0,
        collision_offset_y: -21.0,  // Match shadow (ground footprint)
        base_offset: -24.0,
        visual_offset_y: 10.0,
        destructible: false,
        health: None,
        visual: PropVisual {
            body: PropMeshLayer {
                mesh: meshes.add(Rectangle::new(32.0, 48.0)),
                material: materials.add(Color::srgb(0.45, 0.43, 0.4)),
                offset: Vec3::ZERO,
                rotation: 0.0,
            },
            shadow: Some(PropMeshLayer {
                mesh: shadow_mesh.clone(),
                material: shadow_material.clone(),
                offset: Vec3::new(1.0, -21.0, 0.0),
                rotation: 0.0,
            }),
            details: vec![
                PropMeshLayer {
                    mesh: meshes.add(Rectangle::new(32.0, 6.0)),
                    material: materials.add(Color::srgb(0.55, 0.53, 0.5)),
                    offset: Vec3::new(0.0, 24.0, 0.0),
                    rotation: 0.0,
                },
            ],
        },
    });

    PropRegistry {
        props,
        shadow_mesh,
        shadow_material,
    }
}
