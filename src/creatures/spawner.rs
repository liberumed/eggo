use bevy::prelude::*;
use rand::Rng;

use crate::combat::{create_weapon_arc, CreatureRangeIndicator, Fist, Weapon, WeaponRangeIndicator, WeaponVisual, WeaponVisualMesh, weapon_catalog};
use crate::constants::*;
use crate::core::{CharacterAssets, Health, HitCollider, Loot, Shadow, WalkCollider, YSorted};
use crate::effects::ResourceBall;
use crate::ui::{HeartSprite, HpText};
use super::{Creature, CreatureAnimation, CreatureDefinition, Glowing, Hostile, creature_catalog};

/// Spawn a creature's range indicator as an independent entity
/// This ensures consistent behavior - indicator follows creature but isn't affected by animations
pub fn spawn_creature_range_indicator(
    commands: &mut Commands,
    creature_entity: Entity,
    arc_mesh: Handle<Mesh>,
    indicator_material: Handle<ColorMaterial>,
    position: Vec3,
) {
    commands.spawn((
        WeaponRangeIndicator,
        CreatureRangeIndicator(creature_entity),
        Mesh2d(arc_mesh),
        MeshMaterial2d(indicator_material),
        Transform::from_xyz(position.x, position.y, Z_WEAPON + 0.1),
    ));
}

pub fn spawn_creatures(
    commands: &mut Commands,
    assets: &CharacterAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let mut rng = rand::rng();
    let blob = creature_catalog::blob();
    let hostile_blob = creature_catalog::hostile_blob();
    let world_size = WORLD_SIZE as f32 * GRID_SPACING;
    let min_distance = COLLISION_RADIUS * 6.0;
    let cell_size = min_distance * 1.2;
    let cells_per_axis = (world_size * 2.0 / cell_size) as i32;

    let mut positions: Vec<Vec2> = Vec::new();

    for cx in -cells_per_axis / 2..cells_per_axis / 2 {
        for cy in -cells_per_axis / 2..cells_per_axis / 2 {
            if rng.random_bool(1.0 - CREATURE_SPAWN_CHANCE) {
                continue;
            }

            let base_x = cx as f32 * cell_size;
            let base_y = cy as f32 * cell_size;

            let x = base_x + rng.random_range(-cell_size * 0.4..cell_size * 0.4);
            let y = base_y + rng.random_range(-cell_size * 0.4..cell_size * 0.4);

            // Don't spawn near player start
            if x.abs() < COLLISION_RADIUS * 4.0 && y.abs() < COLLISION_RADIUS * 4.0 {
                continue;
            }

            let pos = Vec2::new(x, y);
            let too_close = positions.iter().any(|p| p.distance(pos) < min_distance);
            if too_close {
                continue;
            }

            positions.push(pos);

            // 15% chance to spawn hostile variant
            let definition = if rng.random_bool(0.15) {
                &hostile_blob
            } else {
                &blob
            };
            spawn_creature(commands, assets, meshes, materials, &mut rng, definition, x, y);
        }
    }
}

fn spawn_creature(
    commands: &mut Commands,
    assets: &CharacterAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    rng: &mut rand::prelude::ThreadRng,
    definition: &CreatureDefinition,
    x: f32,
    y: f32,
) {
    let anim = CreatureAnimation {
        phase: rng.random_range(0.0..std::f32::consts::TAU),
        speed: rng.random_range(2.0..5.0),
        amplitude: rng.random_range(0.04..0.10),
    };

    let is_hostile = rng.random_bool(definition.hostile_chance);
    let is_glowing = !is_hostile && rng.random_bool(definition.glowing_chance);

    let material = if is_hostile {
        assets.hostile_material.clone()
    } else if is_glowing {
        assets.glowing_material.clone()
    } else {
        assets.neutral_material.clone()
    };

    let loot = loop {
        let p = rng.random_bool(definition.loot.philosophy_chance);
        let n = rng.random_bool(definition.loot.nature_chance);
        let w = rng.random_bool(definition.loot.wisdom_chance);
        let count = p as u8 + n as u8 + w as u8;
        if count >= 1 && count <= 3 {
            break Loot { philosophy: p, nature_study: n, wisdom: w };
        }
    };

    // Pre-create fist weapon if hostile (before entering closure)
    let fist_data = if is_hostile {
        let fist = weapon_catalog::fist(meshes, materials);
        let arc_mesh = create_weapon_arc(meshes, &fist);
        Some((fist.visual.clone(), fist, arc_mesh))
    } else {
        None
    };

    // Extract arc_mesh before moving fist_data into closure
    let arc_mesh = fist_data.as_ref().map(|(_, _, mesh)| mesh.clone());
    let fist_only = fist_data.map(|(visual, weapon, _)| (visual, weapon));

    let mut entity_commands = commands.spawn((
        Creature,
        YSorted { base_offset: definition.base_offset },
        WalkCollider {
            radius_x: definition.walk_collider.radius_x,
            radius_y: definition.walk_collider.radius_y,
            offset_y: definition.walk_collider.offset_y,
        },
        HitCollider {
            radius_x: definition.hit_collider.radius_x,
            radius_y: definition.hit_collider.radius_y,
            offset_y: definition.hit_collider.offset_y,
        },
        anim,
        Health(definition.health),
        loot,
        Mesh2d(assets.character_mesh.clone()),
        MeshMaterial2d(material),
        Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(definition.scale)),
    ));

    if is_hostile {
        entity_commands.insert(Hostile { speed: definition.speed });
    }
    if is_glowing {
        entity_commands.insert(Glowing);
    }

    let creature_entity = entity_commands.id();

    entity_commands.with_children(|parent| {
        spawn_creature_children(parent, assets, rng, &loot, fist_only);
    });

    // Spawn range indicator as independent entity
    if let Some(arc_mesh) = arc_mesh {
        spawn_creature_range_indicator(
            commands,
            creature_entity,
            arc_mesh,
            assets.range_indicator_material.clone(),
            Vec3::new(x, y, 0.0),
        );
    }
}

fn spawn_creature_children(
    parent: &mut ChildSpawnerCommands,
    assets: &CharacterAssets,
    #[allow(unused)] rng: &mut rand::prelude::ThreadRng,
    loot: &Loot,
    fist_data: Option<(WeaponVisual, Weapon)>,
) {
    // Shadow
    parent.spawn((
        Shadow::default(),
        Mesh2d(assets.shadow_mesh.clone()),
        MeshMaterial2d(assets.shadow_material.clone()),
        Transform::from_xyz(1.0, -11.0, Z_SHADOW_OFFSET),
    ));
    // Shade
    parent.spawn((
        Mesh2d(assets.shade_mesh.clone()),
        MeshMaterial2d(assets.shade_material.clone()),
        Transform::from_xyz(-3.0, -4.0, Z_CHARACTER_DETAIL),
    ));
    // Shine
    parent.spawn((
        Mesh2d(assets.shine_mesh.clone()),
        MeshMaterial2d(assets.shine_material.clone()),
        Transform::from_xyz(3.0, 6.0, Z_CHARACTER_DETAIL),
    ));
    // Heart display
    parent.spawn((
        HeartSprite,
        Mesh2d(assets.heart_mesh.clone()),
        MeshMaterial2d(assets.heart_material.clone()),
        Transform::from_xyz(-6.0, 19.0, Z_UI_WORLD),
    ));
    parent.spawn((
        HeartSprite,
        Mesh2d(assets.heart_top_mesh.clone()),
        MeshMaterial2d(assets.heart_material.clone()),
        Transform::from_xyz(-7.5, 20.0, Z_UI_WORLD),
    ));
    parent.spawn((
        HeartSprite,
        Mesh2d(assets.heart_top_mesh.clone()),
        MeshMaterial2d(assets.heart_material.clone()),
        Transform::from_xyz(-4.5, 20.0, Z_UI_WORLD),
    ));
    // HP text
    parent.spawn((
        HpText,
        Text2d::new("2"),
        TextFont { font_size: 32.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(2.0, 18.0, Z_UI_WORLD).with_scale(Vec3::splat(0.25)),
    ));

    // Resource balls
    let spawn_ball = |parent: &mut ChildSpawnerCommands, rng: &mut rand::prelude::ThreadRng, ball_material: Handle<ColorMaterial>, mesh: Handle<Mesh>| {
        let angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
        let r: f32 = rng.random_range(0.0..0.6);
        let start_x = angle.cos() * r * 6.0;
        let start_y = angle.sin() * r * 9.0;
        let vel_angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
        let vel_speed: f32 = rng.random_range(8.0..15.0);
        parent.spawn((
            ResourceBall {
                velocity: Vec2::new(vel_angle.cos() * vel_speed, vel_angle.sin() * vel_speed),
            },
            Mesh2d(mesh),
            MeshMaterial2d(ball_material),
            Transform::from_xyz(start_x, start_y, Z_PARTICLE),
        ));
    };

    if loot.philosophy {
        spawn_ball(parent, rng, assets.philosophy_material.clone(), assets.resource_ball_mesh.clone());
    }
    if loot.nature_study {
        spawn_ball(parent, rng, assets.nature_material.clone(), assets.resource_ball_mesh.clone());
    }
    if loot.wisdom {
        spawn_ball(parent, rng, assets.wisdom_material.clone(), assets.resource_ball_mesh.clone());
    }

    // Fist for hostile creatures
    if let Some((fist_visual, fist_weapon)) = fist_data {
        parent.spawn((
            Fist,
            fist_weapon,
            Transform::from_xyz(0.0, 0.0, Z_WEAPON),
            Visibility::default(),
        )).with_children(|fist_holder| {
            fist_holder.spawn((
                WeaponVisualMesh,
                Mesh2d(fist_visual.mesh),
                MeshMaterial2d(fist_visual.material),
                Transform::from_xyz(fist_visual.offset, 0.0, 0.0),
            ));
        });
        // Range indicator spawned as independent entity in spawn_creature()
    }
}
