use bevy::prelude::*;
use rand::Rng;

use crate::components::{StaticCollider, YSorted};
use crate::constants::{COLLISION_RADIUS, GRID_SPACING, WORLD_SIZE, Z_SHADOW_OFFSET};
use super::components::{CrateSprite, Destructible, Prop};
use super::data::{CrateSprites, PropDefinition, PropRegistry, PropType};

/// Spawns a prop from a definition (from registry)
pub fn spawn_prop(
    commands: &mut Commands,
    definition: &PropDefinition,
    position: Vec2,
) {
    let visual = &definition.visual;

    let mut entity = commands.spawn((
        Prop { prop_type: definition.prop_type },
        YSorted { base_offset: definition.base_offset },
        StaticCollider {
            radius_x: definition.collision_radius_x,
            radius_y: definition.collision_radius_y,
            offset_x: definition.collision_offset_x,
            offset_y: definition.collision_offset_y,
        },
        Mesh2d(visual.body.mesh.clone()),
        MeshMaterial2d(visual.body.material.clone()),
        Transform::from_xyz(
            position.x + visual.body.offset.x,
            position.y + definition.visual_offset_y + visual.body.offset.y,
            0.0,
        ),
    ));

    if definition.destructible {
        if let Some(health) = definition.health {
            entity.insert(Destructible { health });
        }
    }

    entity.with_children(|parent| {
        // Shadow
        if let Some(shadow) = &visual.shadow {
            parent.spawn((
                Mesh2d(shadow.mesh.clone()),
                MeshMaterial2d(shadow.material.clone()),
                Transform::from_xyz(shadow.offset.x, shadow.offset.y, Z_SHADOW_OFFSET)
                    .with_rotation(Quat::from_rotation_z(shadow.rotation)),
            ));
        }

        // Details
        for (i, detail) in visual.details.iter().enumerate() {
            let z_offset = 0.001 + (i as f32) * 0.001;
            parent.spawn((
                Mesh2d(detail.mesh.clone()),
                MeshMaterial2d(detail.material.clone()),
                Transform::from_xyz(detail.offset.x, detail.offset.y, z_offset)
                    .with_rotation(Quat::from_rotation_z(detail.rotation)),
            ));
        }
    });
}

/// Spawns a sprite-based crate
pub fn spawn_crate(
    commands: &mut Commands,
    crate_sprites: &CrateSprites,
    registry: &PropRegistry,
    position: Vec2,
) {
    let Some(definition) = registry.get(PropType::Crate) else { return };

    commands.spawn((
        Prop { prop_type: PropType::Crate },
        CrateSprite { damaged: false },
        Destructible { health: 2 },
        YSorted { base_offset: definition.base_offset },
        StaticCollider {
            radius_x: definition.collision_radius_x,
            radius_y: definition.collision_radius_y,
            offset_x: definition.collision_offset_x,
            offset_y: definition.collision_offset_y,
        },
        Sprite {
            image: crate_sprites.texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: crate_sprites.atlas_layout.clone(),
                index: 0,  // Closed crate
            }),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 0.0),
    ));
}

pub fn spawn_world_props(
    commands: &mut Commands,
    registry: &PropRegistry,
    crate_sprites: &CrateSprites,
) {
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
        let prop_type = match rng.random_range(0..4) {
            0 => PropType::Pillar,
            1 => PropType::Barrel,
            2 => PropType::Crate,
            _ => PropType::Pillar,
        };

        match prop_type {
            PropType::Crate => {
                spawn_crate(commands, crate_sprites, registry, pos);
            }
            _ => {
                if let Some(definition) = registry.get(prop_type) {
                    spawn_prop(commands, definition, pos);
                }
            }
        }
    }
}
