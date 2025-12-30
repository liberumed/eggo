use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;

#[derive(Resource)]
pub struct CharacterAssets {
    // Base character mesh
    pub character_mesh: Handle<Mesh>,

    // Character materials
    pub player_material: Handle<ColorMaterial>,
    pub neutral_material: Handle<ColorMaterial>,
    pub hostile_material: Handle<ColorMaterial>,
    pub glowing_material: Handle<ColorMaterial>,
    pub dead_material: Handle<ColorMaterial>,

    // Character details
    pub shadow_mesh: Handle<Mesh>,
    pub shadow_material: Handle<ColorMaterial>,
    pub shine_mesh: Handle<Mesh>,
    pub shine_material: Handle<ColorMaterial>,
    pub shade_mesh: Handle<Mesh>,
    pub shade_material: Handle<ColorMaterial>,

    // Health display
    pub heart_mesh: Handle<Mesh>,
    pub heart_top_mesh: Handle<Mesh>,
    pub heart_material: Handle<ColorMaterial>,

    // Resource balls
    pub resource_ball_mesh: Handle<Mesh>,
    pub philosophy_material: Handle<ColorMaterial>,
    pub nature_material: Handle<ColorMaterial>,
    pub wisdom_material: Handle<ColorMaterial>,

    // Weapons
    pub fist_mesh: Handle<Mesh>,
    pub fist_material: Handle<ColorMaterial>,
    pub knife_blade_mesh: Handle<Mesh>,
    pub knife_handle_mesh: Handle<Mesh>,
    pub blade_material: Handle<ColorMaterial>,
    pub handle_material: Handle<ColorMaterial>,

    // Outline
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<ColorMaterial>,

    // Blood effects
    pub blood_splat_mesh: Handle<Mesh>,
    pub blood_droplet_mesh: Handle<Mesh>,
    pub blood_splat_material: Handle<ColorMaterial>,
    pub blood_droplet_material: Handle<ColorMaterial>,
}

impl CharacterAssets {
    pub fn load(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        let character_mesh = meshes.add(Ellipse::new(10.0, 14.0));

        let player_material = materials.add(Color::srgb(1.0, 0.85, 0.6));
        let neutral_material = materials.add(Color::srgb(0.9, 0.8, 0.5));
        let hostile_material = materials.add(Color::srgb(0.85, 0.25, 0.25));
        let glowing_material = materials.add(Color::srgb(1.0, 0.9, 0.3));
        let dead_material = materials.add(Color::srgb(0.4, 0.4, 0.4));

        let shadow_mesh = meshes.add(Ellipse::new(11.0, 6.0));
        let shadow_material = materials.add(Color::srgba(0.0, 0.0, 0.0, 0.3));

        let shine_mesh = meshes.add(Ellipse::new(3.0, 2.0));
        let shine_material = materials.add(Color::srgba(1.0, 1.0, 1.0, 0.5));

        let shade_mesh = meshes.add(Ellipse::new(4.0, 5.0));
        let shade_material = materials.add(Color::srgba(0.0, 0.0, 0.0, 0.15));

        let heart_mesh = meshes.add(Triangle2d::new(
            Vec2::new(-3.0, 1.0),
            Vec2::new(3.0, 1.0),
            Vec2::new(0.0, -4.0),
        ));
        let heart_top_mesh = meshes.add(Circle::new(2.0));
        let heart_material = materials.add(Color::srgb(1.0, 0.3, 0.3));

        let resource_ball_mesh = meshes.add(Circle::new(2.5));
        let philosophy_material = materials.add(Color::srgb(0.6, 0.3, 0.7));
        let nature_material = materials.add(Color::srgb(0.3, 0.7, 0.3));
        let wisdom_material = materials.add(Color::srgb(0.3, 0.5, 0.9));

        let fist_mesh = meshes.add(Circle::new(3.0));
        let fist_material = materials.add(Color::srgb(0.8, 0.65, 0.5));

        let knife_blade_mesh = meshes.add(Triangle2d::new(
            Vec2::new(0.0, 2.0),
            Vec2::new(0.0, -2.0),
            Vec2::new(9.0, 0.0),
        ));
        let knife_handle_mesh = meshes.add(Rectangle::new(4.0, 3.0));
        let blade_material = materials.add(Color::srgb(0.75, 0.75, 0.8));
        let handle_material = materials.add(Color::srgb(0.45, 0.3, 0.15));

        let outline_mesh = meshes.add(Ellipse::new(11.7, 16.25));
        let outline_material = materials.add(Color::srgba(0.75, 1.0, 0.0, 0.9));

        let blood_splat_mesh = meshes.add(Ellipse::new(4.0, 3.0));
        let blood_droplet_mesh = meshes.add(Circle::new(2.0));
        let blood_splat_material = materials.add(Color::srgb(0.7, 0.0, 0.0));
        let blood_droplet_material = materials.add(Color::srgb(0.9, 0.1, 0.1));

        Self {
            character_mesh,
            player_material,
            neutral_material,
            hostile_material,
            glowing_material,
            dead_material,
            shadow_mesh,
            shadow_material,
            shine_mesh,
            shine_material,
            shade_mesh,
            shade_material,
            heart_mesh,
            heart_top_mesh,
            heart_material,
            resource_ball_mesh,
            philosophy_material,
            nature_material,
            wisdom_material,
            fist_mesh,
            fist_material,
            knife_blade_mesh,
            knife_handle_mesh,
            blade_material,
            handle_material,
            outline_mesh,
            outline_material,
            blood_splat_mesh,
            blood_droplet_mesh,
            blood_splat_material,
            blood_droplet_material,
        }
    }
}

pub fn spawn_player(commands: &mut Commands, assets: &CharacterAssets) {
    commands.spawn((
        Player,
        PlayerAnimation::default(),
        Health(2),
        Mesh2d(assets.character_mesh.clone()),
        MeshMaterial2d(assets.player_material.clone()),
        Transform::from_xyz(0.0, 0.0, Z_PLAYER),
    )).with_children(|parent| {
        // Shadow
        parent.spawn((
            Shadow,
            Mesh2d(assets.shadow_mesh.clone()),
            MeshMaterial2d(assets.shadow_material.clone()),
            Transform::from_xyz(1.0, -11.0, Z_SHADOW - Z_PLAYER),
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
        // Heart sprites
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
            TextFont { font_size: 8.0, ..default() },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            Transform::from_xyz(2.0, 18.0, Z_UI_WORLD),
        ));
        // Knife weapon
        parent.spawn((
            Knife,
            Transform::from_xyz(0.0, 0.0, Z_WEAPON),
            Visibility::default(),
        )).with_children(|knife| {
            knife.spawn((
                Mesh2d(assets.knife_handle_mesh.clone()),
                MeshMaterial2d(assets.handle_material.clone()),
                Transform::from_xyz(11.0, 0.0, 0.0),
            ));
            knife.spawn((
                Mesh2d(assets.knife_blade_mesh.clone()),
                MeshMaterial2d(assets.blade_material.clone()),
                Transform::from_xyz(15.0, 0.0, 0.0),
            ));
        });
    });
}

pub fn spawn_target_outline(commands: &mut Commands, assets: &CharacterAssets) {
    commands.spawn((
        TargetOutline,
        Mesh2d(assets.outline_mesh.clone()),
        MeshMaterial2d(assets.outline_material.clone()),
        Transform::from_xyz(0.0, 0.0, Z_TARGET_OUTLINE),
        Visibility::Hidden,
    ));
}

pub fn spawn_creatures(commands: &mut Commands, assets: &CharacterAssets) {
    let mut rng = rand::rng();
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

            spawn_creature(commands, assets, &mut rng, x, y);
        }
    }
}

fn spawn_creature(
    commands: &mut Commands,
    assets: &CharacterAssets,
    rng: &mut rand::prelude::ThreadRng,
    x: f32,
    y: f32,
) {
    let anim = CreatureAnimation {
        phase: rng.random_range(0.0..std::f32::consts::TAU),
        speed: rng.random_range(2.0..5.0),
        amplitude: rng.random_range(0.04..0.10),
    };

    let is_hostile = rng.random_bool(HOSTILE_CHANCE);
    let is_glowing = !is_hostile && rng.random_bool(GLOWING_CHANCE);

    let material = if is_hostile {
        assets.hostile_material.clone()
    } else if is_glowing {
        assets.glowing_material.clone()
    } else {
        assets.neutral_material.clone()
    };

    let loot = loop {
        let p = rng.random_bool(0.5);
        let n = rng.random_bool(0.5);
        let w = rng.random_bool(0.5);
        let count = p as u8 + n as u8 + w as u8;
        if count >= 1 && count <= 3 {
            break Loot { philosophy: p, nature_study: n, wisdom: w };
        }
    };

    let mut entity_commands = commands.spawn((
        Creature,
        anim,
        Health(2),
        loot,
        Mesh2d(assets.character_mesh.clone()),
        MeshMaterial2d(material),
        Transform::from_xyz(x, y, Z_CREATURE),
    ));

    if is_hostile {
        entity_commands.insert(Hostile);
    }
    if is_glowing {
        entity_commands.insert(Glowing);
    }

    entity_commands.with_children(|parent| {
        spawn_creature_children(parent, assets, rng, &loot, is_hostile);
    });
}

fn spawn_creature_children(
    parent: &mut ChildSpawnerCommands,
    assets: &CharacterAssets,
    rng: &mut rand::prelude::ThreadRng,
    loot: &Loot,
    is_hostile: bool,
) {
    // Shadow
    parent.spawn((
        Shadow,
        Mesh2d(assets.shadow_mesh.clone()),
        MeshMaterial2d(assets.shadow_material.clone()),
        Transform::from_xyz(1.0, -11.0, Z_SHADOW - Z_CREATURE),
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
        TextFont { font_size: 8.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(2.0, 18.0, Z_UI_WORLD),
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
    if is_hostile {
        parent.spawn((
            Fist,
            Transform::from_xyz(0.0, 0.0, Z_WEAPON),
            Visibility::default(),
        )).with_children(|fist_holder| {
            fist_holder.spawn((
                Mesh2d(assets.fist_mesh.clone()),
                MeshMaterial2d(assets.fist_material.clone()),
                Transform::from_xyz(11.0, 0.0, 0.0),
            ));
        });
    }
}

pub fn spawn_background_grid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let dot_mesh = meshes.add(Circle::new(1.0));
    let dot_material = materials.add(Color::srgb(0.35, 0.35, 0.4));

    for x in -WORLD_SIZE..=WORLD_SIZE {
        for y in -WORLD_SIZE..=WORLD_SIZE {
            commands.spawn((
                Mesh2d(dot_mesh.clone()),
                MeshMaterial2d(dot_material.clone()),
                Transform::from_xyz(x as f32 * GRID_SPACING, y as f32 * GRID_SPACING, Z_BACKGROUND),
            ));
        }
    }
}
