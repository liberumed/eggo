mod combat;
mod constants;
mod core;
mod creatures;
mod debug;
mod effects;
mod inventory;
mod levels;
mod player;
mod props;
mod state_machine;
mod ui;
mod world;

use bevy::{image::ImageSamplerDescriptor, prelude::*};
use constants::*;

use core::{CharacterAssets, CorePlugin, GameConfig, GameState, InputBindings};
use levels::{CreatureType, CurrentLevel, LevelBackground, LevelsPlugin, PropType, VoidBackground, WinZone, WinZoneTimer};
use world::{NewGameRequested, WorldConfig};
use creatures::{Creature, CreaturePlugin};
use debug::{
    cleanup_steering_debug, spawn_debug_circles, spawn_steering_debug, spawn_weapon_debug_cones,
    toggle_collision_debug, update_creature_debug_circles, update_debug_visibility,
    update_player_debug_cone, update_steering_debug, DebugConfig,
};
use effects::{BloodParticle, EffectsPlugin, Hitstop, ScreenShake, TargetOutline};
use inventory::{build_item_registry, GroundItem, InventoryPlugin, ItemIcons, ItemId, ItemRegistry};
use player::{
    animate_sprites, load_player_sprite_sheet, update_player_sprite_animation,
    Player, PlayerPlugin, PlayerSpriteSheet, Stats,
};
use props::{build_prop_registry, load_barrel_sprites, load_crate_sprites, load_crate2_sprites, BarrelSprites, CrateSprites, Crate2Sprites, Prop, PropRegistry};
use ui::{
    auto_start_new_game, hide_pause_menu, setup_ui, show_pause_menu, show_victory_menu,
    spawn_key_bindings_panel, toggle_pause_menu, UiPlugin,
};

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
        .init_resource::<WinZoneTimer>()
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.25)))
        .add_systems(Startup, (setup, setup_ui, spawn_key_bindings_panel))
        .add_systems(Update, (
            toggle_pause_menu,
            toggle_collision_debug,
            spawn_debug_circles,
            spawn_weapon_debug_cones,
            spawn_steering_debug,
            update_player_debug_cone,
            update_creature_debug_circles,
            update_steering_debug,
            cleanup_steering_debug,
            update_debug_visibility,
            update_player_sprite_animation,
            animate_sprites,
        ))
        .add_systems(OnEnter(GameState::Playing), spawn_world)
        .add_systems(OnEnter(GameState::Paused), show_pause_menu)
        .add_systems(OnExit(GameState::Paused), hide_pause_menu)
        .add_systems(OnEnter(GameState::Dead), auto_start_new_game)
        .add_systems(OnExit(GameState::Dead), (hide_pause_menu, cleanup_world).chain())
        .add_systems(OnEnter(GameState::Victory), show_victory_menu)
        .add_systems(OnExit(GameState::Victory), (hide_pause_menu, cleanup_world).chain())
        .add_systems(Update, levels::systems::check_win_zone.run_if(in_state(GameState::Playing)))
        .add_plugins((
            state_machine::StateMachinePlugin,
            CorePlugin,
            PlayerPlugin,
            CreaturePlugin,
            EffectsPlugin,
            UiPlugin,
            InventoryPlugin,
            LevelsPlugin,
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
    // Load game config from INI file (with fallback to defaults)
    let game_config = GameConfig::load_from_file("config.ini");

    // Camera
    commands.spawn((
        Camera2d,
        Transform::default().with_scale(Vec3::splat(1.0 / PIXEL_SCALE)),
    ));

    // Load assets
    let character_assets = CharacterAssets::load(&mut meshes, &mut materials);
    let player_sprite_sheet = load_player_sprite_sheet(&asset_server, &mut texture_atlas_layouts);
    let item_registry = build_item_registry(&game_config, &mut meshes, &mut materials);
    let prop_registry = build_prop_registry(&mut meshes, &mut materials);
    let crate_sprites = load_crate_sprites(&asset_server, &mut texture_atlas_layouts);
    let crate2_sprites = load_crate2_sprites(&asset_server, &mut texture_atlas_layouts);
    let barrel_sprites = load_barrel_sprites(&asset_server, &mut texture_atlas_layouts);
    let item_icons = load_item_icons(&asset_server);

    // Insert resources
    commands.insert_resource(game_config);
    commands.insert_resource(character_assets);
    commands.insert_resource(player_sprite_sheet);
    commands.insert_resource(item_registry);
    commands.insert_resource(prop_registry);
    commands.insert_resource(crate_sprites);
    commands.insert_resource(crate2_sprites);
    commands.insert_resource(barrel_sprites);
    commands.insert_resource(item_icons);

    next_state.set(GameState::Playing);
}

fn load_item_icons(asset_server: &AssetServer) -> ItemIcons {
    let mut icons = ItemIcons::default();
    icons.icons.insert(ItemId::RustyKnife, asset_server.load("sprites/items/knife.png"));
    icons.ground_icons.insert(ItemId::RustyKnife, asset_server.load("sprites/items/knife_ground.png"));
    icons.icons.insert(ItemId::WoodenStick, asset_server.load("sprites/items/stick.png"));
    icons.ground_icons.insert(ItemId::WoodenStick, asset_server.load("sprites/items/stick_ground.png"));
    icons.icons.insert(ItemId::Mushroom, asset_server.load("sprites/items/mushroom.png"));
    icons.ground_icons.insert(ItemId::Mushroom, asset_server.load("sprites/items/mushroom_ground.png"));
    icons
}

fn spawn_world(
    mut commands: Commands,
    config: Res<GameConfig>,
    character_assets: Res<CharacterAssets>,
    player_sprite_sheet: Res<PlayerSpriteSheet>,
    item_registry: Res<ItemRegistry>,
    item_icons: Res<ItemIcons>,
    prop_registry: Res<PropRegistry>,
    crate_sprites: Res<CrateSprites>,
    crate2_sprites: Res<Crate2Sprites>,
    barrel_sprites: Res<BarrelSprites>,
    mut current_level: ResMut<CurrentLevel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<Entity, With<Player>>,
) {
    // Skip if player already exists (resuming from pause)
    if player_query.iter().next().is_some() {
        return;
    }

    // Load level data
    current_level.load("assets/levels/corridor.ron");
    let level = current_level.data.as_ref().expect("Level should be loaded");

    // Spawn level background (void and corridor)
    levels::spawn_level_background(&mut commands, level, &mut meshes, &mut materials);

    // Spawn win zone if level has one
    if let Some(win_zone) = &level.win_zone {
        levels::spawn_win_zone(&mut commands, win_zone.position, win_zone.radius, &mut meshes, &mut materials);
    }

    // Spawn pits
    for pit in &level.pits {
        levels::spawn_pit(&mut commands, pit.position, pit.radius, pit.edge_radius, &mut meshes, &mut materials);
    }

    // Spawn player at level's spawn position
    player::spawn_player(&mut commands, &config, &character_assets, &player_sprite_sheet, &mut meshes, &mut materials, level.player_spawn);
    player::spawn_target_outline(&mut commands, &character_assets);

    for spawn in &level.items {
        player::spawn_ground_item(&mut commands, &character_assets, &item_registry, &item_icons, spawn.item_id, spawn.quantity, spawn.position);
    }

    for spawn in &level.creatures {
        match spawn.creature {
            CreatureType::Goblin => {
                creatures::spawn_goblin(&mut commands, &config, &character_assets, &player_sprite_sheet, &mut meshes, &mut materials, spawn.position);
            }
        }
    }

    for spawn in &level.props {
        match spawn.prop {
            PropType::Pillar => props::spawn_pillar(&mut commands, &prop_registry, spawn.position),
            PropType::Barrel => props::spawn_barrel(&mut commands, &barrel_sprites, &prop_registry, spawn.position),
            PropType::Crate => props::spawn_crate(&mut commands, &crate_sprites, &prop_registry, spawn.position),
            PropType::Crate2 => props::spawn_crate2(&mut commands, &crate2_sprites, &prop_registry, spawn.position),
        }
    }
}

fn cleanup_world(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Player>, With<Creature>, With<BloodParticle>, With<TargetOutline>, With<GroundItem>, With<Prop>, With<LevelBackground>, With<VoidBackground>, With<WinZone>)>>,
    mut stats: ResMut<Stats>,
    mut current_level: ResMut<CurrentLevel>,
    mut win_zone_timer: ResMut<WinZoneTimer>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    stats.philosophy = 0;
    stats.nature_study = 0;
    stats.wisdom = 0;
    current_level.data = None;
    win_zone_timer.0 = 0.0;
}
