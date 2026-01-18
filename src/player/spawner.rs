use bevy::prelude::*;
use rand::Rng;

use crate::combat::{create_half_circle_arc, Equipment, PlayerRangeIndicator, WeaponRangeIndicator};
use crate::inventory::weapons::{Drawn, PlayerWeapon, WeaponVisualMesh, weapon_catalog};
use crate::constants::*;
use crate::core::{CharacterAssets, Health, Shadow, WalkCollider, HitCollider, YSorted};
use crate::effects::TargetOutline;
use crate::inventory::{EquippedWeaponId, GroundItem, GroundItemBob, Inventory, ItemIcons, ItemId, ItemRegistry, Pickupable};
use crate::state_machine::StateMachine;
use crate::ui::{HeartSprite, HpText};
use crate::levels::BoundToLevel;
use super::{ComboState, FacingDirection, MovementInput, Player, PlayerAnimation, PlayerSpriteSheet, PlayerState, SpriteAnimation};

pub fn spawn_ground_item(
    commands: &mut Commands,
    assets: &CharacterAssets,
    registry: &ItemRegistry,
    item_icons: &ItemIcons,
    item_id: ItemId,
    quantity: u32,
    position: Vec2,
) {
    let mut rng = rand::rng();
    let item = registry.items.get(&item_id).expect("Item not found in registry");

    commands
        .spawn((
            GroundItem { item_id, quantity },
            GroundItemBob {
                phase: rng.random_range(0.0..std::f32::consts::TAU),
                hovered: false,
            },
            Pickupable,
            Transform::from_xyz(position.x, position.y, Z_PARTICLE),
        ))
        .with_children(|parent| {
            // Glow effect underneath
            parent.spawn((
                Mesh2d(assets.item_glow_mesh.clone()),
                MeshMaterial2d(assets.item_glow_material.clone()),
                Transform::from_xyz(0.0, 0.0, -0.1),
            ));

            // Use ground sprite if available, otherwise mesh visuals
            if let Some(ground_icon) = item_icons.ground_icons.get(&item_id) {
                parent.spawn((
                    Sprite::from_image(ground_icon.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                ));
            } else {
                for (mesh, material, offset) in &item.ground_visual.meshes {
                    parent.spawn((
                        Mesh2d(mesh.clone()),
                        MeshMaterial2d(material.clone()),
                        Transform::from_translation(*offset),
                    ));
                }
            }
        });
}

pub fn spawn_player(
    commands: &mut Commands,
    assets: &CharacterAssets,
    sprite_sheet: &PlayerSpriteSheet,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    spawn_pos: Vec2,
) {
    // Player starts with sword
    let weapon = weapon_catalog::sword(meshes, materials);
    let weapon_visual = weapon.visual.clone();

    // Create inventory with sword and stick
    let mut inventory = Inventory::default();
    inventory.try_add(ItemId::Sword, 1);
    inventory.try_add(ItemId::WoodenStick, 1);

    // Get initial animation data (start facing down)
    let initial_anim = sprite_sheet.animations.get("idle_down").unwrap_or_else(|| {
        sprite_sheet.animations.values().next().expect("No animations in sprite sheet")
    });

    commands.spawn((
        // Core identity
        Player,
        BoundToLevel,
        StateMachine::<PlayerState>::default(),
        // Physics/rendering
        YSorted { base_offset: -24.0 },  // Feet position for 64x64 sprite
        WalkCollider { radius_x: 8.0, radius_y: 4.0, offset_y: -4.0 },  // At feet
        HitCollider::ellipse_vertical(5.0, 10.0, 14.0),  // Centered on body
        // Animation state
        PlayerAnimation::default(),
        MovementInput::default(),
        FacingDirection::default(),
        ComboState::default(),
        SpriteAnimation::new("idle_down", initial_anim.frame_duration_ms),
        // Combat/inventory
        Health(10),
        Equipment::default(),
        inventory,
        EquippedWeaponId(ItemId::Sword),
    )).insert((
        // Sprite and transform (separate insert to avoid tuple limit)
        Sprite::from_atlas_image(
            initial_anim.texture.clone(),
            TextureAtlas {
                layout: initial_anim.atlas_layout.clone(),
                index: initial_anim.start_index,
            },
        ),
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 0.0),
    )).with_children(|parent| {
        // Shadow - right under feet
        parent.spawn((
            Shadow { base_scale: Vec2::new(0.6, 0.5) },  // Shadow for sprite
            Mesh2d(assets.shadow_mesh.clone()),
            MeshMaterial2d(assets.shadow_material.clone()),
            Transform::from_xyz(0.0, -3.0, Z_SHADOW_OFFSET),
        ));
        // Heart sprites
        parent.spawn((
            HeartSprite,
            Mesh2d(assets.heart_mesh.clone()),
            MeshMaterial2d(assets.heart_material.clone()),
            Transform::from_xyz(-6.0, 36.0, Z_UI_WORLD),
        ));
        parent.spawn((
            HeartSprite,
            Mesh2d(assets.heart_top_mesh.clone()),
            MeshMaterial2d(assets.heart_material.clone()),
            Transform::from_xyz(-7.5, 37.0, Z_UI_WORLD),
        ));
        parent.spawn((
            HeartSprite,
            Mesh2d(assets.heart_top_mesh.clone()),
            MeshMaterial2d(assets.heart_material.clone()),
            Transform::from_xyz(-4.5, 37.0, Z_UI_WORLD),
        ));
        // HP text
        parent.spawn((
            HpText,
            Text2d::new("2"),
            TextFont { font_size: 32.0, ..default() },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            Transform::from_xyz(2.0, 35.0, Z_UI_WORLD).with_scale(Vec3::splat(0.25)),
        ));
        // Default weapon (already drawn)
        let weapon_pos = Vec3::new(12.0, 5.0, Z_WEAPON);
        parent.spawn((
            PlayerWeapon,
            Drawn,
            weapon.clone(),
            Transform::from_translation(weapon_pos),
        )).with_children(|weapon_parent| {
            weapon_parent.spawn((
                WeaponVisualMesh,
                Mesh2d(weapon_visual.mesh),
                MeshMaterial2d(weapon_visual.material),
                Transform::from_xyz(weapon_visual.offset, 0.0, 0.0),
            ));
        });
        // Range indicator - centered on player body for half-circle attacks
        let arc_mesh = create_half_circle_arc(meshes, weapon.range());
        parent.spawn((
            WeaponRangeIndicator,
            PlayerRangeIndicator,
            Mesh2d(arc_mesh),
            MeshMaterial2d(assets.range_indicator_material.clone()),
            Transform::from_xyz(0.0, ATTACK_CENTER_OFFSET_Y, Z_WEAPON + 0.1),
        ));
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
