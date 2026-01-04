mod combat;
mod constants;
mod core;
mod creatures;
mod debug;
mod effects;
mod inventory;
mod player;
mod props;
mod spawners;
mod ui;

use bevy::{image::ImageSamplerDescriptor, prelude::*};
use constants::*;

// Domain module imports
use core::CorePlugin;
use creatures::CreaturePlugin;
use effects::EffectsPlugin;
use inventory::InventoryPlugin;
use player::PlayerPlugin;
use ui::UiPlugin;
use core::{GameState, InputBindings, NewGameRequested, WorldConfig};
use creatures::Creature;
use debug::DebugConfig;
use effects::{Hitstop, ScreenShake, BloodParticle, TargetOutline};
use inventory::{build_item_registry, GroundItem, ItemIcons, ItemId, ItemRegistry};
use player::{animate_sprites, Player, PlayerSpriteSheet, Stats, load_player_sprite_sheet, update_player_sprite_animation};
use props::{CrateSprites, Prop, PropRegistry, build_prop_registry, load_crate_sprites};
use spawners::CharacterAssets;
use ui::{
    auto_start_new_game, hide_pause_menu, show_pause_menu, spawn_key_bindings_panel, toggle_pause_menu,
    HotbarUI, HotbarSlot, HotbarSlotIcon, HotbarSlotCount,
    InventoryPanel, InventorySlotUI, InventorySlotIcon, InventorySlotCount,
    WeaponInfoPanel, WeaponNameText, WeaponDamageText, WeaponSpeedText,
    WeaponRangeText, WeaponConeText, WeaponKnockbackText, WeaponTypeText,
    GameMenu, MenuTitle, ResumeButton, MenuNewGameButton, ExitButton,
    PhilosophyCounter, NatureStudyCounter, WisdomCounter,
};
use debug::{spawn_debug_circles, spawn_weapon_debug_cones, toggle_collision_debug, update_creature_debug_circles, update_debug_visibility, update_player_debug_cone};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Eggo".to_string(),
                    resolution: ((320.0 * PIXEL_SCALE) as u32, (240.0 * PIXEL_SCALE) as u32).into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor::nearest(),
            }))
        .init_resource::<Stats>()
        .init_resource::<WorldConfig>()
        .init_resource::<NewGameRequested>()
        .init_resource::<InputBindings>()
        .init_resource::<Hitstop>()
        .init_resource::<ScreenShake>()
        .init_resource::<DebugConfig>()
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.25)))
        .add_systems(Startup, (setup, setup_ui, spawn_key_bindings_panel))
        .add_systems(Update, (toggle_pause_menu, toggle_collision_debug, spawn_debug_circles, spawn_weapon_debug_cones, update_player_debug_cone, update_creature_debug_circles, update_debug_visibility, update_player_sprite_animation, animate_sprites))
        .add_systems(OnEnter(GameState::Playing), spawn_world)
        .add_systems(OnEnter(GameState::Paused), show_pause_menu)
        .add_systems(OnExit(GameState::Paused), hide_pause_menu)
        .add_systems(OnEnter(GameState::Dead), auto_start_new_game)
        .add_systems(OnExit(GameState::Dead), (hide_pause_menu, cleanup_world).chain())
        .add_plugins((
            CorePlugin,
            PlayerPlugin,
            CreaturePlugin,
            EffectsPlugin,
            UiPlugin,
            InventoryPlugin,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Camera
    commands.spawn((
        Camera2d,
        Transform::default().with_scale(Vec3::splat(1.0 / PIXEL_SCALE)),
    ));

    // Load assets
    let character_assets = CharacterAssets::load(&mut meshes, &mut materials);

    // Load player sprite sheet
    let player_sprite_sheet = load_player_sprite_sheet(&asset_server, &mut texture_atlas_layouts);
    commands.insert_resource(player_sprite_sheet);

    // Build registries
    let item_registry = build_item_registry(&mut meshes, &mut materials);
    let prop_registry = build_prop_registry(&mut meshes, &mut materials);
    let crate_sprites = load_crate_sprites(&asset_server, &mut texture_atlas_layouts);

    // Load item icons (UI icons and ground sprites)
    let mut item_icons = ItemIcons::default();
    item_icons.icons.insert(ItemId::RustyKnife, asset_server.load("sprites/items/knife.png"));
    item_icons.ground_icons.insert(ItemId::RustyKnife, asset_server.load("sprites/items/knife_ground.png"));
    item_icons.icons.insert(ItemId::WoodenStick, asset_server.load("sprites/items/stick.png"));
    item_icons.ground_icons.insert(ItemId::WoodenStick, asset_server.load("sprites/items/stick_ground.png"));
    item_icons.icons.insert(ItemId::Mushroom, asset_server.load("sprites/items/mushroom.png"));
    item_icons.ground_icons.insert(ItemId::Mushroom, asset_server.load("sprites/items/mushroom_ground.png"));

    // Spawn background (static, doesn't need reset)
    spawners::spawn_background_grid(&mut commands, &mut meshes, &mut materials);

    // Insert resources for later use
    commands.insert_resource(character_assets);
    commands.insert_resource(item_registry);
    commands.insert_resource(prop_registry);
    commands.insert_resource(crate_sprites);
    commands.insert_resource(item_icons);

    // Transition to Playing state (triggers spawn_world)
    next_state.set(GameState::Playing);
}

fn spawn_world(
    mut commands: Commands,
    character_assets: Res<CharacterAssets>,
    player_sprite_sheet: Res<PlayerSpriteSheet>,
    item_registry: Res<ItemRegistry>,
    item_icons: Res<ItemIcons>,
    prop_registry: Res<PropRegistry>,
    crate_sprites: Res<CrateSprites>,
    config: Res<WorldConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<Entity, With<Player>>,
) {
    // Skip if player already exists (resuming from pause)
    if player_query.iter().next().is_some() {
        return;
    }

    spawners::spawn_player(&mut commands, &character_assets, &player_sprite_sheet, &mut meshes, &mut materials);
    spawners::spawn_target_outline(&mut commands, &character_assets);
    spawners::spawn_creatures(&mut commands, &character_assets, &mut meshes, &mut materials);
    spawners::spawn_world_props(&mut commands, &prop_registry, &crate_sprites);

    for (item_id, quantity, pos) in &config.starting_items {
        spawners::spawn_ground_item(&mut commands, &character_assets, &item_registry, &item_icons, *item_id, *quantity, *pos);
    }
}

fn cleanup_world(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Player>, With<Creature>, With<BloodParticle>, With<TargetOutline>, With<GroundItem>, With<Prop>)>>,
    mut stats: ResMut<Stats>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    stats.philosophy = 0;
    stats.nature_study = 0;
    stats.wisdom = 0;
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Auto,
            padding: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(40.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            ..default()
        })
        .with_children(|parent| {
            spawn_stat_circle(parent, Color::srgb(0.6, 0.3, 0.7), PhilosophyCounter);
            spawn_stat_circle(parent, Color::srgb(0.3, 0.7, 0.3), NatureStudyCounter);
            spawn_stat_circle(parent, Color::srgb(0.3, 0.5, 0.9), WisdomCounter);
        });

    commands
        .spawn((
            WeaponInfoPanel,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.12, 0.85)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                WeaponNameText,
                Text::new("Rusty Knife"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.85, 0.5)),
            ));
            spawn_weapon_stat(parent, "DMG", "1", WeaponDamageText);
            spawn_weapon_stat(parent, "SPD", "3", WeaponSpeedText);
            spawn_weapon_stat(parent, "RCH", "3", WeaponRangeText);
            spawn_weapon_stat(parent, "ARC", "1", WeaponConeText);
            spawn_weapon_stat(parent, "IMP", "3", WeaponKnockbackText);
            spawn_weapon_stat(parent, "TYPE", "Slash", WeaponTypeText);
        });

    // Hotbar UI (bottom center)
    commands
        .spawn((
            HotbarUI,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(130.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-130.0)),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            for i in 0..5 {
                parent
                    .spawn((
                        HotbarSlot(i),
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.22, 0.9)),
                        BorderColor::all(Color::srgb(0.4, 0.4, 0.45)),
                    ))
                    .with_children(|slot| {
                        // Item icon - Auto mode preserves aspect ratio
                        slot.spawn((
                            HotbarSlotIcon(i),
                            ImageNode::default(),
                            Node {
                                max_width: Val::Px(40.0),
                                max_height: Val::Px(40.0),
                                ..default()
                            },
                            Visibility::Hidden,
                        ));
                        // Number label (top-left)
                        slot.spawn((
                            Text::new((i + 1).to_string()),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(2.0),
                                left: Val::Px(4.0),
                                ..default()
                            },
                        ));
                        // Stack count (bottom-right)
                        slot.spawn((
                            HotbarSlotCount(i),
                            Text::new(""),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(2.0),
                                right: Val::Px(4.0),
                                ..default()
                            },
                        ));
                    });
            }
        });

    // Inventory Panel (Tab toggle)
    commands
        .spawn((
            InventoryPanel,
            Button,
            Interaction::None,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                right: Val::Px(20.0),
                width: Val::Px(320.0),
                height: Val::Px(240.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.12, 0.14, 0.95)),
            BorderRadius::all(Val::Px(8.0)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("INVENTORY"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
            ));

            // Inventory grid (2 rows of 5)
            for row in 0..2 {
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|row_node| {
                        for col in 0..5 {
                            let index = row * 5 + col;
                            row_node
                                .spawn((
                                    InventorySlotUI(index),
                                    Button,
                                    Node {
                                        width: Val::Px(48.0),
                                        height: Val::Px(48.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        overflow: Overflow::clip(),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.2, 0.2, 0.22, 1.0)),
                                    BorderColor::all(Color::srgb(0.4, 0.4, 0.45)),
                                ))
                                .with_children(|slot| {
                                    // Item icon - Auto mode preserves aspect ratio
                                    slot.spawn((
                                        InventorySlotIcon(index),
                                        ImageNode::default(),
                                        Node {
                                            max_width: Val::Px(40.0),
                                            max_height: Val::Px(40.0),
                                            ..default()
                                        },
                                        Visibility::Hidden,
                                    ));
                                    // Stack count (bottom-right)
                                    slot.spawn((
                                        InventorySlotCount(index),
                                        Text::new(""),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                        Node {
                                            position_type: PositionType::Absolute,
                                            bottom: Val::Px(2.0),
                                            right: Val::Px(4.0),
                                            ..default()
                                        },
                                    ));
                                });
                        }
                    });
            }

            // Instructions
            parent.spawn((
                Text::new("Right Click item to use/equip"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });

    // Game Menu (unified pause/death menu)
    commands.spawn((
        GameMenu,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(30.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        Visibility::Hidden,
    )).with_children(|parent| {
        // Title (changes between "PAUSED" and "YOU DIED")
        parent.spawn((
            MenuTitle,
            Text::new("PAUSED"),
            TextFont {
                font_size: 64.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));

        // Resume button (hidden when dead)
        parent.spawn((
            ResumeButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(30.0), Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.5, 0.3)),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("RESUME"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });

        // New Game button
        parent.spawn((
            MenuNewGameButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(30.0), Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.35)),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("NEW GAME"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });

        // Exit button
        parent.spawn((
            ExitButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(30.0), Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.5, 0.3, 0.3)),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("EXIT"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
    });
}

fn spawn_weapon_stat<T: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    marker: T,
) {
    parent
        .spawn(Node {
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
            row.spawn((
                marker,
                Text::new(value),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

fn spawn_stat_circle<T: Component>(parent: &mut ChildSpawnerCommands, color: Color, marker: T) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Node {
                    width: Val::Px(50.0),
                    height: Val::Px(50.0),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(color),
                BorderColor::all(Color::srgb(0.7, 0.7, 0.7)),
                BorderRadius::MAX,
            ));

            col.spawn((
                Text::new("0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
                marker,
            ));
        });
}
