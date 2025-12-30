mod components;
mod constants;
mod resources;
mod spawners;
mod systems;

use bevy::prelude::*;
use components::*;
use constants::*;
use resources::Stats;
use spawners::CharacterAssets;
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Eggo".to_string(),
                resolution: ((320.0 * PIXEL_SCALE) as u32, (240.0 * PIXEL_SCALE) as u32).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<Stats>()
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.25)))
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(Update, (
            // Movement
            move_player,
            apply_knockback,
            // Animation
            animate_player,
            animate_creatures,
            animate_knife_swing,
            animate_fist_swing,
            animate_death,
            // Combat
            knife_attack,
            aim_knife,
            hostile_ai,
            hostile_fist_aim,
            hostile_attack,
        ))
        .add_systems(Update, (
            // Camera & UI
            camera_follow,
            update_counters,
            update_hp_text,
            stabilize_text_rotation,
            show_death_screen,
            handle_new_game_button,
            // Effects
            animate_resource_balls,
            animate_magnetized_balls,
            animate_blood,
            // Status
            update_stun,
            update_despawn_timer,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera2d,
        Transform::default().with_scale(Vec3::splat(1.0 / PIXEL_SCALE)),
    ));

    // Load all character assets
    let assets = CharacterAssets::load(&mut meshes, &mut materials);

    // Spawn game entities
    spawners::spawn_player(&mut commands, &assets);
    spawners::spawn_target_outline(&mut commands, &assets);
    spawners::spawn_creatures(&mut commands, &assets);
    spawners::spawn_background_grid(&mut commands, &mut meshes, &mut materials);

    // Insert assets as resource for later use
    commands.insert_resource(assets);
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

    commands.spawn((
        DeathScreen,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(40.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        Visibility::Hidden,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("YOU DIED"),
            TextFont {
                font_size: 64.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.2, 0.2)),
        ));
        parent.spawn((
            NewGameButton,
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
