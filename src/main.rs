use bevy::prelude::*;
use rand::Rng;

const PIXEL_SCALE: f32 = 4.0;

const Z_BACKGROUND: f32 = -10.0;
const Z_SHADOW: f32 = -5.0;
const Z_DEAD_BODY: f32 = -2.0;
const Z_WORLD_EGG: f32 = 0.0;
const Z_PLAYER: f32 = 1.0;
const Z_EGG_DETAIL: f32 = 0.1;
const Z_WEAPON: f32 = 0.2;
const Z_RESOURCE_BALL: f32 = 0.3;
const Z_HP_DISPLAY: f32 = 5.0;
const Z_BLOOD: f32 = 3.0;
const Z_AIM_OUTLINE: f32 = -0.5;

const EGG_SPEED: f32 = 75.0;
const AGGRESSIVE_EGG_SPEED: f32 = 30.0;
const AGGRESSIVE_SIGHT_RANGE: f32 = 100.0;
const FIST_ATTACK_RANGE: f32 = EGG_RADIUS * 1.8;
const KNOCKBACK_FORCE: f32 = 250.0;
const GRID_SIZE: i32 = 20;
const GRID_SPACING: f32 = 25.0;
const EGG_RADIUS: f32 = 14.0;
const KNIFE_RANGE: f32 = EGG_RADIUS * 3.5;

#[derive(Component)]
struct Egg;

#[derive(Component)]
struct Knife;

#[derive(Component, Default)]
struct EggAnimation {
    time: f32,
    velocity: Vec2,
}

#[derive(Component)]
struct WorldEgg;

#[derive(Component)]
struct WorldEggAnimation {
    phase: f32,
    speed: f32,
    amplitude: f32,
}

#[derive(Component)]
struct EggShadow;

#[derive(Component)]
struct Dead;

#[derive(Component)]
struct DespawnTimer(f32);

#[derive(Component)]
struct AimOutline;

#[derive(Component)]
struct Stunned(f32);

#[derive(Component)]
struct DeathScreen;

#[derive(Component)]
struct NewGameButton;

#[derive(Component)]
struct BloodParticle {
    velocity: Vec2,
    lifetime: f32,
}

#[derive(Resource)]
struct BloodMaterials {
    splat: Handle<ColorMaterial>,
    droplet: Handle<ColorMaterial>,
}

#[derive(Resource)]
struct FistAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

#[derive(Resource)]
struct BloodMeshes {
    splat: Handle<Mesh>,
    droplet: Handle<Mesh>,
}

#[derive(Component)]
struct GlowingEgg;

#[derive(Component)]
struct Aggressive;

#[derive(Component)]
struct Fist;

#[derive(Component)]
struct FistSwing {
    timer: f32,
}

#[derive(Component)]
struct Knockback {
    velocity: Vec2,
    timer: f32,
}

#[derive(Component)]
struct Health(i32);

#[derive(Component, Clone, Copy)]
struct ResourceReward {
    philosophy: bool,
    nature_study: bool,
    wisdom: bool,
}

#[derive(Resource)]
struct DeadMaterial(Handle<ColorMaterial>);

#[derive(Component)]
struct HpText;

#[derive(Component)]
struct HeartSprite;

#[derive(Component)]
struct ResourceBall {
    velocity: Vec2,
}

#[derive(Component)]
struct MagnetizedBall;


#[derive(Component)]
struct KnifeSwing {
    timer: f32,
    base_angle: f32,
}

#[derive(Component)]
struct DeathAnimation {
    timer: f32,
    stage: u8,
}

#[derive(Resource, Default)]
struct Stats {
    philosophy: u32,
    nature_study: u32,
    wisdom: u32,
}


#[derive(Component)]
struct PhilosophyCounter;

#[derive(Component)]
struct NatureStudyCounter;

#[derive(Component)]
struct WisdomCounter;

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
            move_egg,
            apply_knockback,
            animate_egg,
            animate_world_eggs,
            aggressive_egg_ai,
            aggressive_fist_aim,
            aggressive_egg_attack,
            animate_fist_swing,
            animate_knife_swing,
            animate_death,
            knife_attack,
            aim_knife,
        ))
        .add_systems(Update, (
            camera_follow,
            update_counters,
            update_hp_text,
            stabilize_text_rotation,
            animate_resource_balls,
            animate_magnetized_balls,
            update_stun,
            show_death_screen,
            handle_new_game_button,
            animate_blood,
            update_despawn_timer,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Transform::default().with_scale(Vec3::splat(1.0 / PIXEL_SCALE)),
    ));

    let egg_mesh = meshes.add(Ellipse::new(10.0, 14.0));
    let player_material = materials.add(Color::srgb(1.0, 0.85, 0.6));
    let world_egg_material = materials.add(Color::srgb(0.9, 0.8, 0.5));
    let aggressive_material = materials.add(Color::srgb(0.85, 0.25, 0.25));
    let glowing_material = materials.add(Color::srgb(1.0, 0.9, 0.3));
    let dead_material = materials.add(Color::srgb(0.4, 0.4, 0.4));

    let shadow_mesh = meshes.add(Ellipse::new(11.0, 6.0));
    let shadow_material = materials.add(Color::srgba(0.0, 0.0, 0.0, 0.3));

    let shine_mesh = meshes.add(Ellipse::new(3.0, 2.0));
    let shine_material = materials.add(Color::srgba(1.0, 1.0, 1.0, 0.5));

    let shade_mesh = meshes.add(Ellipse::new(4.0, 5.0));
    let shade_material = materials.add(Color::srgba(0.0, 0.0, 0.0, 0.15));

    let fist_mesh = meshes.add(Circle::new(3.0));
    let fist_material = materials.add(Color::srgb(0.8, 0.65, 0.5));

    let resource_ball_mesh = meshes.add(Circle::new(2.5));
    let philosophy_ball_material = materials.add(Color::srgb(0.6, 0.3, 0.7));
    let nature_ball_material = materials.add(Color::srgb(0.3, 0.7, 0.3));
    let wisdom_ball_material = materials.add(Color::srgb(0.3, 0.5, 0.9));

    let heart_mesh = meshes.add(Triangle2d::new(
        Vec2::new(-3.0, 1.0),
        Vec2::new(3.0, 1.0),
        Vec2::new(0.0, -4.0),
    ));
    let heart_top_mesh = meshes.add(Circle::new(2.0));
    let heart_material = materials.add(Color::srgb(1.0, 0.3, 0.3));

    let outline_mesh = meshes.add(Ellipse::new(11.7, 16.25));
    let outline_material = materials.add(Color::srgba(0.75, 1.0, 0.0, 0.9));

    commands.spawn((
        AimOutline,
        Mesh2d(outline_mesh),
        MeshMaterial2d(outline_material),
        Transform::from_xyz(0.0, 0.0, Z_AIM_OUTLINE),
        Visibility::Hidden,
    ));

    commands.insert_resource(BloodMeshes {
        splat: meshes.add(Ellipse::new(4.0, 3.0)),
        droplet: meshes.add(Circle::new(2.0)),
    });
    commands.insert_resource(BloodMaterials {
        splat: materials.add(Color::srgb(0.7, 0.0, 0.0)),
        droplet: materials.add(Color::srgb(0.9, 0.1, 0.1)),
    });
    commands.insert_resource(FistAssets {
        mesh: fist_mesh.clone(),
        material: fist_material.clone(),
    });

    commands.insert_resource(DeadMaterial(dead_material));

    let knife_blade = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 2.0),
        Vec2::new(0.0, -2.0),
        Vec2::new(9.0, 0.0),
    ));
    let knife_handle = meshes.add(Rectangle::new(4.0, 3.0));
    let blade_material = materials.add(Color::srgb(0.75, 0.75, 0.8));
    let handle_material = materials.add(Color::srgb(0.45, 0.3, 0.15));

    commands.spawn((
        Egg,
        EggAnimation::default(),
        Health(2),
        Mesh2d(egg_mesh.clone()),
        MeshMaterial2d(player_material),
        Transform::from_xyz(0.0, 0.0, Z_PLAYER),
    )).with_children(|parent| {
        parent.spawn((
            EggShadow,
            Mesh2d(shadow_mesh.clone()),
            MeshMaterial2d(shadow_material.clone()),
            Transform::from_xyz(1.0, -11.0, Z_SHADOW - Z_PLAYER),
        ));
        parent.spawn((
            Mesh2d(shade_mesh.clone()),
            MeshMaterial2d(shade_material.clone()),
            Transform::from_xyz(-3.0, -4.0, Z_EGG_DETAIL),
        ));
        parent.spawn((
            Mesh2d(shine_mesh.clone()),
            MeshMaterial2d(shine_material.clone()),
            Transform::from_xyz(3.0, 6.0, Z_EGG_DETAIL),
        ));
        parent.spawn((
            HeartSprite,
            Mesh2d(heart_mesh.clone()),
            MeshMaterial2d(heart_material.clone()),
            Transform::from_xyz(-6.0, 19.0, Z_HP_DISPLAY),
        ));
        parent.spawn((
            HeartSprite,
            Mesh2d(heart_top_mesh.clone()),
            MeshMaterial2d(heart_material.clone()),
            Transform::from_xyz(-7.5, 20.0, Z_HP_DISPLAY),
        ));
        parent.spawn((
            HeartSprite,
            Mesh2d(heart_top_mesh.clone()),
            MeshMaterial2d(heart_material.clone()),
            Transform::from_xyz(-4.5, 20.0, Z_HP_DISPLAY),
        ));
        parent.spawn((
            HpText,
            Text2d::new("2"),
            TextFont { font_size: 8.0, ..default() },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            Transform::from_xyz(2.0, 18.0, Z_HP_DISPLAY),
        ));
        parent.spawn((
            Knife,
            Transform::from_xyz(0.0, 0.0, Z_WEAPON),
            Visibility::default(),
        )).with_children(|knife| {
            knife.spawn((
                Mesh2d(knife_handle),
                MeshMaterial2d(handle_material),
                Transform::from_xyz(11.0, 0.0, 0.0),
            ));
            knife.spawn((
                Mesh2d(knife_blade),
                MeshMaterial2d(blade_material),
                Transform::from_xyz(15.0, 0.0, 0.0),
            ));
        });
    });

    spawn_world_eggs(
        &mut commands,
        egg_mesh,
        world_egg_material,
        aggressive_material,
        glowing_material,
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
        philosophy_ball_material,
        nature_ball_material,
        wisdom_ball_material,
        fist_mesh,
        fist_material,
    );

    let dot_mesh = meshes.add(Circle::new(1.0));
    let dot_material = materials.add(Color::srgb(0.35, 0.35, 0.4));

    for x in -GRID_SIZE..=GRID_SIZE {
        for y in -GRID_SIZE..=GRID_SIZE {
            commands.spawn((
                Mesh2d(dot_mesh.clone()),
                MeshMaterial2d(dot_material.clone()),
                Transform::from_xyz(x as f32 * GRID_SPACING, y as f32 * GRID_SPACING, Z_BACKGROUND),
            ));
        }
    }
}

fn spawn_world_eggs(
    commands: &mut Commands,
    egg_mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    aggressive_material: Handle<ColorMaterial>,
    glowing_material: Handle<ColorMaterial>,
    shadow_mesh: Handle<Mesh>,
    shadow_material: Handle<ColorMaterial>,
    shine_mesh: Handle<Mesh>,
    shine_material: Handle<ColorMaterial>,
    shade_mesh: Handle<Mesh>,
    shade_material: Handle<ColorMaterial>,
    heart_mesh: Handle<Mesh>,
    heart_top_mesh: Handle<Mesh>,
    heart_material: Handle<ColorMaterial>,
    resource_ball_mesh: Handle<Mesh>,
    philosophy_ball_material: Handle<ColorMaterial>,
    nature_ball_material: Handle<ColorMaterial>,
    wisdom_ball_material: Handle<ColorMaterial>,
    fist_mesh: Handle<Mesh>,
    fist_material: Handle<ColorMaterial>,
) {
    let mut rng = rand::rng();
    let world_size = GRID_SIZE as f32 * GRID_SPACING;
    let min_egg_distance = EGG_RADIUS * 6.0;
    let cell_size = min_egg_distance * 1.2;
    let cells_per_axis = (world_size * 2.0 / cell_size) as i32;

    let mut egg_positions: Vec<Vec2> = Vec::new();

    for cx in -cells_per_axis / 2..cells_per_axis / 2 {
        for cy in -cells_per_axis / 2..cells_per_axis / 2 {
            if rng.random_bool(0.4) {
                continue;
            }

            let base_x = cx as f32 * cell_size;
            let base_y = cy as f32 * cell_size;

            let x = base_x + rng.random_range(-cell_size * 0.4..cell_size * 0.4);
            let y = base_y + rng.random_range(-cell_size * 0.4..cell_size * 0.4);

            if x.abs() < EGG_RADIUS * 4.0 && y.abs() < EGG_RADIUS * 4.0 {
                continue;
            }

            let pos = Vec2::new(x, y);
            let too_close = egg_positions.iter().any(|p| p.distance(pos) < min_egg_distance);
            if too_close {
                continue;
            }

            egg_positions.push(pos);

            let anim = WorldEggAnimation {
                phase: rng.random_range(0.0..std::f32::consts::TAU),
                speed: rng.random_range(2.0..5.0),
                amplitude: rng.random_range(0.04..0.10),
            };

            let is_aggressive = rng.random_bool(0.15);
            let is_glowing = !is_aggressive && rng.random_bool(0.3);

            let egg_material = if is_aggressive {
                aggressive_material.clone()
            } else if is_glowing {
                glowing_material.clone()
            } else {
                material.clone()
            };

            let reward = loop {
                let p = rng.random_bool(0.5);
                let n = rng.random_bool(0.5);
                let w = rng.random_bool(0.5);
                let count = p as u8 + n as u8 + w as u8;
                if count >= 1 && count <= 3 {
                    break ResourceReward { philosophy: p, nature_study: n, wisdom: w };
                }
            };


            let mut entity_commands = commands.spawn((
                WorldEgg,
                anim,
                Health(2),
                reward,
                Mesh2d(egg_mesh.clone()),
                MeshMaterial2d(egg_material),
                Transform::from_xyz(x, y, Z_WORLD_EGG),
            ));

            if is_aggressive {
                entity_commands.insert(Aggressive);
            }
            if is_glowing {
                entity_commands.insert(GlowingEgg);
            }

            entity_commands.with_children(|parent| {
                parent.spawn((
                    EggShadow,
                    Mesh2d(shadow_mesh.clone()),
                    MeshMaterial2d(shadow_material.clone()),
                    Transform::from_xyz(1.0, -11.0, Z_SHADOW - Z_WORLD_EGG),
                ));
                parent.spawn((
                    Mesh2d(shade_mesh.clone()),
                    MeshMaterial2d(shade_material.clone()),
                    Transform::from_xyz(-3.0, -4.0, Z_EGG_DETAIL),
                ));
                parent.spawn((
                    Mesh2d(shine_mesh.clone()),
                    MeshMaterial2d(shine_material.clone()),
                    Transform::from_xyz(3.0, 6.0, Z_EGG_DETAIL),
                ));

                parent.spawn((
                    HeartSprite,
                    Mesh2d(heart_mesh.clone()),
                    MeshMaterial2d(heart_material.clone()),
                    Transform::from_xyz(-6.0, 19.0, Z_HP_DISPLAY),
                ));
                parent.spawn((
                    HeartSprite,
                    Mesh2d(heart_top_mesh.clone()),
                    MeshMaterial2d(heart_material.clone()),
                    Transform::from_xyz(-7.5, 20.0, Z_HP_DISPLAY),
                ));
                parent.spawn((
                    HeartSprite,
                    Mesh2d(heart_top_mesh.clone()),
                    MeshMaterial2d(heart_material.clone()),
                    Transform::from_xyz(-4.5, 20.0, Z_HP_DISPLAY),
                ));
                parent.spawn((
                    HpText,
                    Text2d::new("2"),
                    TextFont { font_size: 8.0, ..default() },
                    TextColor(Color::srgb(1.0, 1.0, 1.0)),
                    Transform::from_xyz(2.0, 18.0, Z_HP_DISPLAY),
                ));

                let spawn_ball = |parent: &mut ChildSpawnerCommands, rng: &mut rand::prelude::ThreadRng, ball_material: Handle<ColorMaterial>| {
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
                        Mesh2d(resource_ball_mesh.clone()),
                        MeshMaterial2d(ball_material),
                        Transform::from_xyz(start_x, start_y, Z_RESOURCE_BALL),
                    ));
                };

                if reward.philosophy {
                    spawn_ball(parent, &mut rng, philosophy_ball_material.clone());
                }
                if reward.nature_study {
                    spawn_ball(parent, &mut rng, nature_ball_material.clone());
                }
                if reward.wisdom {
                    spawn_ball(parent, &mut rng, wisdom_ball_material.clone());
                }

                if is_aggressive {
                    parent.spawn((
                        Fist,
                        Transform::from_xyz(0.0, 0.0, Z_WEAPON),
                        Visibility::default(),
                    )).with_children(|fist_holder| {
                        fist_holder.spawn((
                            Mesh2d(fist_mesh.clone()),
                            MeshMaterial2d(fist_material.clone()),
                            Transform::from_xyz(11.0, 0.0, 0.0),
                        ));
                    });
                }
            });
        }
    }
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

fn knife_attack(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, (With<Egg>, Without<Dead>)>,
    knife_query: Query<(Entity, &Transform), With<Knife>>,
    knife_swing_query: Query<&KnifeSwing>,
    mut world_eggs_query: Query<(Entity, &Transform, &mut Health, Option<&Aggressive>), (With<WorldEgg>, Without<Dead>, Without<DeathAnimation>)>,
    blood_meshes: Res<BloodMeshes>,
    blood_materials: Res<BloodMaterials>,
    fist_assets: Res<FistAssets>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    if knife_swing_query.iter().next().is_some() {
        return;
    }

    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    let knife_dir = if let Ok((knife_entity, knife_transform)) = knife_query.single() {
        let (_, angle) = knife_transform.rotation.to_axis_angle();
        let base_angle = if knife_transform.rotation.z < 0.0 { -angle } else { angle };
        commands.entity(knife_entity).insert(KnifeSwing { timer: 0.0, base_angle });
        Vec2::new(base_angle.cos(), base_angle.sin())
    } else {
        Vec2::X
    };

    let mut rng = rand::rng();

    for (entity, egg_transform, mut health, aggressive) in &mut world_eggs_query {
        let egg_pos = Vec2::new(egg_transform.translation.x, egg_transform.translation.y);

        if player_pos.distance(egg_pos) < KNIFE_RANGE {
            health.0 -= 1;
            commands.entity(entity).insert(Stunned(1.0));

            let particle_count = if health.0 <= 0 { 25 } else { 12 };
            for i in 0..particle_count {
                let spread = rng.random_range(-0.8..0.8);
                let speed = rng.random_range(80.0..200.0);
                let angle = knife_dir.y.atan2(knife_dir.x) + spread;
                let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);

                let is_splat = i % 3 == 0;
                let (mesh, material) = if is_splat {
                    (blood_meshes.splat.clone(), blood_materials.splat.clone())
                } else {
                    (blood_meshes.droplet.clone(), blood_materials.droplet.clone())
                };

                let offset = Vec2::new(
                    rng.random_range(-5.0..5.0),
                    rng.random_range(-5.0..5.0),
                );

                commands.spawn((
                    BloodParticle {
                        velocity: vel,
                        lifetime: rng.random_range(0.4..1.2),
                    },
                    Mesh2d(mesh),
                    MeshMaterial2d(material),
                    Transform::from_xyz(egg_pos.x + offset.x, egg_pos.y + offset.y, Z_BLOOD)
                        .with_rotation(Quat::from_rotation_z(rng.random_range(0.0..std::f32::consts::TAU))),
                ));
            }

            if health.0 <= 0 {
                commands.entity(entity).insert(DeathAnimation {
                    timer: 0.0,
                    stage: 0,
                });
            } else if aggressive.is_none() {
                // Make non-aggressive egg become hostile when hit
                commands.entity(entity).insert(Aggressive);

                // Spawn a fist for the newly hostile egg
                let fist_entity = commands.spawn((
                    Fist,
                    Transform::from_xyz(0.0, 0.0, Z_WEAPON),
                    Visibility::default(),
                )).with_children(|fist_holder| {
                    fist_holder.spawn((
                        Mesh2d(fist_assets.mesh.clone()),
                        MeshMaterial2d(fist_assets.material.clone()),
                        Transform::from_xyz(11.0, 0.0, 0.0),
                    ));
                }).id();

                commands.entity(entity).add_child(fist_entity);
            }
        }
    }
}

fn aim_knife(
    player_query: Query<&Transform, With<Egg>>,
    mut knife_query: Query<&mut Transform, (With<Knife>, Without<Egg>, Without<KnifeSwing>, Without<AimOutline>)>,
    world_eggs_query: Query<&Transform, (With<WorldEgg>, Without<Dead>, Without<Egg>, Without<Knife>, Without<AimOutline>)>,
    mut outline_query: Query<(&mut Transform, &mut Visibility), (With<AimOutline>, Without<Knife>, Without<Egg>, Without<WorldEgg>)>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let Ok(mut knife_transform) = knife_query.single_mut() else { return };
    let Ok((mut outline_transform, mut outline_visibility)) = outline_query.single_mut() else { return };

    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    let closest = world_eggs_query
        .iter()
        .min_by(|a, b| {
            let pos_a = Vec2::new(a.translation.x, a.translation.y);
            let pos_b = Vec2::new(b.translation.x, b.translation.y);
            pos_a.distance(player_pos)
                .partial_cmp(&pos_b.distance(player_pos))
                .unwrap()
        });

    if let Some(target) = closest {
        let target_pos = Vec2::new(target.translation.x, target.translation.y);
        let dir = target_pos - player_pos;
        let angle = dir.y.atan2(dir.x);
        knife_transform.rotation = Quat::from_rotation_z(angle);

        outline_transform.translation.x = target_pos.x;
        outline_transform.translation.y = target_pos.y;
        outline_transform.scale = target.scale;
        outline_transform.rotation = target.rotation;
        *outline_visibility = Visibility::Inherited;
    } else {
        *outline_visibility = Visibility::Hidden;
    }
}

fn animate_knife_swing(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut KnifeSwing)>,
) {
    for (entity, mut transform, mut swing) in &mut query {
        swing.timer += time.delta_secs();

        let t = swing.timer;
        let duration = 0.4;

        if t < duration {
            let progress = t / duration;

            let (thrust_scale, rotation_offset) = if progress < 0.15 {
                let p = progress / 0.15;
                (1.0 - p * 0.2, p * 0.15)
            } else if progress < 0.4 {
                let p = (progress - 0.15) / 0.25;
                (0.8 + p * 0.8, 0.15 - p * 0.15)
            } else if progress < 0.6 {
                let p = (progress - 0.4) / 0.2;
                (1.6 - p * 0.1, -p * 0.1)
            } else {
                let p = (progress - 0.6) / 0.4;
                (1.5 - p * 0.5, -0.1 + p * 0.1)
            };

            transform.scale = Vec3::new(thrust_scale, 1.0, 1.0);
            transform.rotation = Quat::from_rotation_z(swing.base_angle + rotation_offset);
        } else {
            transform.scale = Vec3::ONE;
            transform.rotation = Quat::from_rotation_z(swing.base_angle);
            commands.entity(entity).remove::<KnifeSwing>();
        }
    }
}

fn animate_death(
    mut commands: Commands,
    time: Res<Time>,
    dead_material: Res<DeadMaterial>,
    mut stats: ResMut<Stats>,
    mut query: Query<(Entity, &Transform, &mut DeathAnimation, &ResourceReward, &Children), With<WorldEgg>>,
    ball_query: Query<&Transform, With<ResourceBall>>,
) {
    let mut balls_to_magnetize: Vec<(Entity, Vec3)> = Vec::new();

    for (entity, transform, mut death, reward, children) in &mut query {
        death.timer += time.delta_secs();
        let t = death.timer;

        match death.stage {
            0 => {
                let shake = (t * 60.0).sin() * 0.15 * (1.0 - t * 2.0).max(0.0);
                let expand = 1.0 + t * 0.5;

                commands.entity(entity).insert(Transform {
                    translation: Vec3::new(transform.translation.x, transform.translation.y, Z_DEAD_BODY),
                    rotation: Quat::from_rotation_z(shake),
                    scale: Vec3::new(expand, expand * 0.9, 1.0),
                });

                if t > 0.4 {
                    death.stage = 1;
                    death.timer = 0.0;

                    let parent_pos = transform.translation;
                    let child_list: Vec<Entity> = children.iter().collect();
                    for child in child_list {
                        if let Ok(ball_transform) = ball_query.get(child) {
                            let world_pos = Vec3::new(
                                parent_pos.x + ball_transform.translation.x,
                                parent_pos.y + ball_transform.translation.y,
                                Z_RESOURCE_BALL,
                            );
                            balls_to_magnetize.push((child, world_pos));
                        }
                    }

                    commands.entity(entity).insert((
                        Dead,
                        DespawnTimer(3.0),
                        MeshMaterial2d(dead_material.0.clone()),
                    ));

                    if reward.philosophy { stats.philosophy += 1; }
                    if reward.nature_study { stats.nature_study += 1; }
                    if reward.wisdom { stats.wisdom += 1; }
                }
            }
            1 => {
                let squish = 1.2 - t * 0.8;
                let squash_x = squish.max(0.6) * 1.3;
                let squash_y = (2.0 - squish).min(1.4) * 0.5;

                commands.entity(entity).insert(Transform {
                    translation: Vec3::new(transform.translation.x, transform.translation.y, Z_DEAD_BODY),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(squash_x, squash_y, 1.0),
                });

                if t > 0.3 {
                    commands.entity(entity).remove::<DeathAnimation>();
                    commands.entity(entity).insert(Transform {
                        translation: Vec3::new(transform.translation.x, transform.translation.y, Z_DEAD_BODY),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(1.0, 0.4, 1.0),
                    });
                }
            }
            _ => {}
        }
    }

    for (child_entity, world_pos) in balls_to_magnetize {
        commands.entity(child_entity).insert((
            MagnetizedBall,
            DespawnTimer(3.0),
        ));
        commands.entity(child_entity).remove_parent_in_place();
        commands.entity(child_entity).insert(Transform::from_xyz(world_pos.x, world_pos.y, world_pos.z));
    }
}

fn move_egg(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut EggAnimation), (With<Egg>, Without<Dead>)>,
    world_eggs_query: Query<(&Transform, Option<&Dead>), (With<WorldEgg>, Without<Egg>)>,
) {
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let collision_distance = EGG_RADIUS * 1.5;

    for (mut transform, mut anim) in &mut player_query {
        if direction == Vec2::ZERO {
            anim.velocity = Vec2::ZERO;
            continue;
        }

        let dir_normalized = direction.normalize();
        let velocity = dir_normalized * EGG_SPEED * time.delta_secs();
        let new_pos = Vec2::new(
            transform.translation.x + velocity.x,
            transform.translation.y + velocity.y,
        );

        let mut blocked_x = false;
        let mut blocked_y = false;

        for (world_egg, dead) in &world_eggs_query {
            if dead.is_some() {
                continue;
            }

            let world_pos = Vec2::new(world_egg.translation.x, world_egg.translation.y);

            let test_x = Vec2::new(new_pos.x, transform.translation.y);
            if test_x.distance(world_pos) < collision_distance {
                blocked_x = true;
            }

            let test_y = Vec2::new(transform.translation.x, new_pos.y);
            if test_y.distance(world_pos) < collision_distance {
                blocked_y = true;
            }
        }

        let mut actual_velocity = Vec2::ZERO;
        if !blocked_x {
            transform.translation.x = new_pos.x;
            actual_velocity.x = velocity.x;
        }
        if !blocked_y {
            transform.translation.y = new_pos.y;
            actual_velocity.y = velocity.y;
        }
        anim.velocity = actual_velocity;
    }
}

fn animate_egg(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut EggAnimation), With<Egg>>,
) {
    for (mut transform, mut anim) in &mut query {
        anim.time += time.delta_secs();
        let t = anim.time;

        let is_moving = anim.velocity.length() > 0.1;

        if is_moving {
            let speed_factor = (anim.velocity.length() / (EGG_SPEED * time.delta_secs())).min(1.0);
            let bounce = (t * 20.0).sin() * 0.12 * speed_factor;
            let funk = (t * 7.0).cos() * 0.04 * speed_factor;
            let squash_x = 1.0 + bounce + funk;
            let squash_y = 1.0 - bounce * 0.7 - funk * 0.5;

            transform.scale = Vec3::new(squash_x, squash_y, 1.0);

            let tilt = anim.velocity.x * 0.002;
            let dance_tilt = (t * 15.0).sin() * 0.03 * speed_factor;
            transform.rotation = Quat::from_rotation_z(-tilt + dance_tilt);
        } else {
            let swing = (t * 3.5).sin();
            let swing_snap = swing.signum() * swing.abs().powf(0.6) * 0.12;

            let groove = ((t * 4.0).sin() * 0.5 + 0.5).powf(1.5) * 0.08;
            let funk = (t * 1.2).cos() * 0.05;
            let jazz = ((t * 5.0).sin() * (t * 1.5).cos()) * 0.03;

            transform.scale.x = 1.0 + groove - funk * 0.5 + jazz;
            transform.scale.y = 1.0 - groove * 0.7 + funk * 0.3 - jazz * 0.5;
            transform.rotation = Quat::from_rotation_z(swing_snap + jazz * 1.5);
        }
    }
}

fn animate_world_eggs(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &WorldEggAnimation), (With<WorldEgg>, Without<Dead>)>,
) {
    let t = time.elapsed_secs();

    for (mut transform, anim) in &mut query {
        let phase = anim.phase;
        let speed = anim.speed;
        let amp = anim.amplitude * 2.5;

        let swing = (t * speed + phase).sin();
        let swing_snap = swing.signum() * swing.abs().powf(0.7) * amp * 1.5;

        let groove = ((t * speed * 2.0 + phase).sin() * 0.5 + 0.5).powf(1.5);
        let bounce = groove * amp * 0.8;

        let funk = (t * speed * 0.5 + phase * 1.3).cos() * amp * 0.6;
        let jazz = ((t * speed * 3.0 + phase).sin() * (t * speed * 0.7).cos()) * amp * 0.3;

        transform.scale.x = 1.0 + bounce - funk * 0.5 + jazz;
        transform.scale.y = 1.0 - bounce * 0.7 + funk * 0.3 - jazz * 0.5;
        transform.rotation = Quat::from_rotation_z(swing_snap + jazz * 2.0);
    }
}

fn aggressive_egg_ai(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Egg>, Without<WorldEgg>)>,
    mut egg_queries: ParamSet<(
        Query<(Entity, &Transform), (With<WorldEgg>, Without<Dead>)>,
        Query<(Entity, &mut Transform), (With<Aggressive>, Without<Dead>, Without<Egg>, Without<Stunned>)>,
    )>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    let egg_positions: Vec<(Entity, Vec2)> = egg_queries
        .p0()
        .iter()
        .map(|(e, t)| (e, Vec2::new(t.translation.x, t.translation.y)))
        .collect();

    let collision_dist = EGG_RADIUS * 1.8;

    for (entity, mut transform) in egg_queries.p1().iter_mut() {
        let egg_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let distance = player_pos.distance(egg_pos);

        if distance < AGGRESSIVE_SIGHT_RANGE && distance > EGG_RADIUS {
            let dir = (player_pos - egg_pos).normalize();
            let movement = dir * AGGRESSIVE_EGG_SPEED * time.delta_secs();
            let new_pos = egg_pos + movement;

            let blocked = egg_positions.iter().any(|(other_entity, other_pos)| {
                *other_entity != entity && new_pos.distance(*other_pos) < collision_dist
            });

            if !blocked {
                transform.translation.x = new_pos.x;
                transform.translation.y = new_pos.y;
            }
        }
    }
}

fn aggressive_fist_aim(
    player_query: Query<&Transform, (With<Egg>, Without<WorldEgg>)>,
    aggressive_query: Query<(&Transform, &Children), (With<Aggressive>, Without<Dead>)>,
    mut fist_query: Query<&mut Transform, (With<Fist>, Without<Aggressive>, Without<Egg>, Without<FistSwing>)>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    for (egg_transform, children) in &aggressive_query {
        let egg_pos = Vec2::new(egg_transform.translation.x, egg_transform.translation.y);

        for child in children.iter() {
            if let Ok(mut fist_transform) = fist_query.get_mut(child) {
                let dir = player_pos - egg_pos;
                let angle = dir.y.atan2(dir.x);
                fist_transform.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}

fn aggressive_egg_attack(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &mut Health), (With<Egg>, Without<WorldEgg>, Without<Dead>)>,
    aggressive_query: Query<(&Transform, &Children), (With<Aggressive>, Without<Dead>, Without<Stunned>)>,
    fist_query: Query<Entity, (With<Fist>, Without<FistSwing>)>,
    knockback_query: Query<&Knockback>,
) {
    let Ok((player_entity, player_transform, mut player_health)) = player_query.single_mut() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    if knockback_query.get(player_entity).is_ok() {
        return;
    }

    for (egg_transform, children) in &aggressive_query {
        let egg_pos = Vec2::new(egg_transform.translation.x, egg_transform.translation.y);
        let distance = player_pos.distance(egg_pos);

        if distance < FIST_ATTACK_RANGE {
            for child in children.iter() {
                if let Ok(fist_entity) = fist_query.get(child) {
                    commands.entity(fist_entity).insert(FistSwing { timer: 0.0 });

                    player_health.0 -= 1;

                    if player_health.0 <= 0 {
                        commands.entity(player_entity).insert(Dead);
                    }

                    let knockback_dir = (player_pos - egg_pos).normalize();
                    commands.entity(player_entity).insert(Knockback {
                        velocity: knockback_dir * KNOCKBACK_FORCE,
                        timer: 0.0,
                    });
                    return;
                }
            }
        }
    }
}

fn animate_fist_swing(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut FistSwing)>,
) {
    for (entity, mut transform, mut swing) in &mut query {
        swing.timer += time.delta_secs();
        let t = swing.timer;
        let duration = 0.3;

        if t < duration {
            let progress = t / duration;
            let punch_scale = if progress < 0.3 {
                1.0 + (progress / 0.3) * 0.5
            } else {
                1.5 - 0.5 * ((progress - 0.3) / 0.7)
            };
            transform.scale = Vec3::splat(punch_scale);
        } else {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<FistSwing>();
        }
    }
}

fn apply_knockback(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Knockback), With<Egg>>,
) {
    for (entity, mut transform, mut knockback) in &mut query {
        knockback.timer += time.delta_secs();

        let decay = (1.0 - knockback.timer * 3.0).max(0.0);
        let movement = knockback.velocity * decay * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;

        if knockback.timer > 0.3 {
            commands.entity(entity).remove::<Knockback>();
        }
    }
}

fn camera_follow(
    egg_query: Query<&Transform, With<Egg>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Egg>)>,
) {
    let Ok(egg_transform) = egg_query.single() else { return };
    let Ok(mut camera_transform) = camera_query.single_mut() else { return };

    camera_transform.translation.x = egg_transform.translation.x;
    camera_transform.translation.y = egg_transform.translation.y;
}

fn update_counters(
    stats: Res<Stats>,
    mut philosophy_query: Query<&mut Text, (With<PhilosophyCounter>, Without<NatureStudyCounter>, Without<WisdomCounter>)>,
    mut nature_query: Query<&mut Text, (With<NatureStudyCounter>, Without<PhilosophyCounter>, Without<WisdomCounter>)>,
    mut wisdom_query: Query<&mut Text, (With<WisdomCounter>, Without<PhilosophyCounter>, Without<NatureStudyCounter>)>,
) {
    if let Ok(mut text) = philosophy_query.single_mut() {
        **text = stats.philosophy.to_string();
    }
    if let Ok(mut text) = nature_query.single_mut() {
        **text = stats.nature_study.to_string();
    }
    if let Ok(mut text) = wisdom_query.single_mut() {
        **text = stats.wisdom.to_string();
    }
}

fn update_hp_text(
    health_query: Query<(&Health, &Children, Option<&Dead>), Or<(With<Egg>, With<WorldEgg>)>>,
    mut text_query: Query<(&mut Text2d, &mut Visibility), With<HpText>>,
    mut heart_query: Query<&mut Visibility, (With<HeartSprite>, Without<HpText>)>,
) {
    for (health, children, dead) in &health_query {
        for child in children.iter() {
            if let Ok((mut text, mut visibility)) = text_query.get_mut(child) {
                if dead.is_some() {
                    *visibility = Visibility::Hidden;
                } else {
                    **text = health.0.to_string();
                }
            }
            if let Ok(mut visibility) = heart_query.get_mut(child) {
                if dead.is_some() {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}

fn stabilize_text_rotation(
    parent_query: Query<(&Transform, &Children), Or<(With<Egg>, With<WorldEgg>)>>,
    mut text_query: Query<&mut Transform, (With<HpText>, Without<Egg>, Without<WorldEgg>)>,
) {
    for (parent_transform, children) in &parent_query {
        let inverse_rotation = parent_transform.rotation.inverse();
        for child in children.iter() {
            if let Ok(mut text_transform) = text_query.get_mut(child) {
                text_transform.rotation = inverse_rotation;
            }
        }
    }
}

fn animate_resource_balls(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ResourceBall), Without<MagnetizedBall>>,
) {
    let dt = time.delta_secs();
    let egg_a = 6.5;
    let egg_b = 10.0;

    for (mut transform, mut ball) in &mut query {
        let mut pos = Vec2::new(transform.translation.x, transform.translation.y);
        pos += ball.velocity * dt;

        let norm_x = pos.x / egg_a;
        let norm_y = pos.y / egg_b;
        let dist_sq = norm_x * norm_x + norm_y * norm_y;

        if dist_sq > 1.0 {
            let normal = Vec2::new(pos.x / (egg_a * egg_a), pos.y / (egg_b * egg_b)).normalize();
            ball.velocity = ball.velocity - 2.0 * ball.velocity.dot(normal) * normal;
            let scale = 1.0 / dist_sq.sqrt();
            pos.x *= scale;
            pos.y *= scale;
        }

        ball.velocity.y -= 5.0 * dt;

        let wobble = (time.elapsed_secs() * 3.0 + pos.x * 0.5).sin() * 2.0;
        ball.velocity.x += wobble * dt;

        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

fn animate_magnetized_balls(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, (With<Egg>, Without<MagnetizedBall>)>,
    mut query: Query<(Entity, &mut Transform, &mut DespawnTimer), (With<MagnetizedBall>, With<ResourceBall>)>,
) {
    let dt = time.delta_secs();
    let player_pos = player_query.single().map(|t| Vec2::new(t.translation.x, t.translation.y)).unwrap_or(Vec2::ZERO);

    for (entity, mut transform, mut timer) in &mut query {
        timer.0 -= dt;

        if timer.0 <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let ball_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let dir = (player_pos - ball_pos).normalize_or_zero();
        let elapsed = 3.0 - timer.0;
        let speed = 80.0 + elapsed * 40.0;
        let new_pos = ball_pos + dir * speed * dt;

        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;

        let fade = (timer.0 / 3.0).powf(0.5);
        transform.scale = Vec3::splat(fade.max(0.1));
    }
}

fn update_stun(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Stunned)>,
) {
    for (entity, mut stunned) in &mut query {
        stunned.0 -= time.delta_secs();
        if stunned.0 <= 0.0 {
            commands.entity(entity).remove::<Stunned>();
        }
    }
}

fn show_death_screen(
    player_query: Query<&Dead, With<Egg>>,
    mut death_screen_query: Query<&mut Visibility, With<DeathScreen>>,
) {
    if player_query.iter().next().is_some() {
        if let Ok(mut visibility) = death_screen_query.single_mut() {
            *visibility = Visibility::Inherited;
        }
    }
}

fn handle_new_game_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NewGameButton>)>,
    mut exit: MessageWriter<AppExit>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Ok(exe) = std::env::current_exe() {
                let _ = std::process::Command::new(exe).spawn();
            }
            exit.write(AppExit::Success);
        }
    }
}

fn animate_blood(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut BloodParticle)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle) in &mut query {
        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        particle.velocity.y -= 150.0 * dt;
        particle.velocity *= 0.98;

        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        let fade = (particle.lifetime / 0.5).min(1.0);
        transform.scale = Vec3::splat(fade * 0.8 + 0.2);

        transform.rotation *= Quat::from_rotation_z(dt * 3.0);
    }
}

fn update_despawn_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DespawnTimer), With<Dead>>,
) {
    let dt = time.delta_secs();
    for (entity, mut timer) in &mut query {
        timer.0 -= dt;
        if timer.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
