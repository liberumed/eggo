use bevy::prelude::*;
use rand::Rng;

const EGG_SPEED: f32 = 300.0;
const GRID_SIZE: i32 = 20;
const GRID_SPACING: f32 = 100.0;
const EGG_RADIUS: f32 = 55.0;

#[derive(Component)]
struct Egg;

#[derive(Component)]
struct WorldEgg;

#[derive(Component)]
struct Cracked;

#[derive(Resource, Default)]
struct Stats {
    philosophy: u32,
    nature_study: u32,
    wisdom: u32,
}

#[derive(Resource)]
struct EggMaterials {
    cracked: Handle<ColorMaterial>,
}

#[derive(Component)]
struct PhilosophyCounter;

#[derive(Component)]
struct NatureStudyCounter;

#[derive(Component)]
struct WisdomCounter;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Stats>()
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(Update, (move_egg, camera_follow, update_counters))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let egg_mesh = meshes.add(Ellipse::new(40.0, 55.0));
    let player_material = materials.add(Color::srgb(1.0, 0.9, 0.7));
    let world_egg_material = materials.add(Color::srgb(0.95, 0.85, 0.65));
    let cracked_material = materials.add(Color::srgb(0.75, 0.7, 0.6));

    commands.insert_resource(EggMaterials {
        cracked: cracked_material,
    });

    commands.spawn((
        Egg,
        Mesh2d(egg_mesh.clone()),
        MeshMaterial2d(player_material),
    ));

    let dot_mesh = meshes.add(Circle::new(4.0));
    let dot_material = materials.add(Color::srgb(0.85, 0.85, 0.85));

    for x in -GRID_SIZE..=GRID_SIZE {
        for y in -GRID_SIZE..=GRID_SIZE {
            commands.spawn((
                Mesh2d(dot_mesh.clone()),
                MeshMaterial2d(dot_material.clone()),
                Transform::from_xyz(x as f32 * GRID_SPACING, y as f32 * GRID_SPACING, -1.0),
            ));
        }
    }

    spawn_world_eggs(&mut commands, egg_mesh, world_egg_material);
}

fn spawn_world_eggs(
    commands: &mut Commands,
    egg_mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
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
            commands.spawn((
                WorldEgg,
                Mesh2d(egg_mesh.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(x, y, 0.0),
            ));
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
            ..default()
        })
        .with_children(|parent| {
            spawn_stat_circle(parent, Color::srgb(0.6, 0.3, 0.7), PhilosophyCounter);
            spawn_stat_circle(parent, Color::srgb(0.3, 0.7, 0.3), NatureStudyCounter);
            spawn_stat_circle(parent, Color::srgb(0.9, 0.8, 0.2), WisdomCounter);
        });
}

fn spawn_stat_circle<T: Component>(parent: &mut ChildBuilder, color: Color, marker: T) {
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
                BorderColor(Color::srgb(0.2, 0.2, 0.2)),
                BorderRadius::MAX,
            ));

            col.spawn((
                Text::new("0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.2, 0.2)),
                marker,
            ));
        });
}

fn move_egg(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    egg_materials: Res<EggMaterials>,
    mut stats: ResMut<Stats>,
    mut player_query: Query<&mut Transform, With<Egg>>,
    world_eggs_query: Query<(Entity, &Transform, Option<&Cracked>), (With<WorldEgg>, Without<Egg>)>,
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

    if direction == Vec2::ZERO {
        return;
    }

    direction = direction.normalize();
    let collision_distance = EGG_RADIUS * 1.5;

    for mut transform in &mut player_query {
        let velocity = direction * EGG_SPEED * time.delta_secs();
        let new_pos = Vec2::new(
            transform.translation.x + velocity.x,
            transform.translation.y + velocity.y,
        );

        let mut blocked_x = false;
        let mut blocked_y = false;
        let mut eggs_to_crack: Vec<Entity> = Vec::new();

        for (entity, world_egg, cracked) in &world_eggs_query {
            let world_pos = Vec2::new(world_egg.translation.x, world_egg.translation.y);
            let is_cracked = cracked.is_some();

            let test_x = Vec2::new(new_pos.x, transform.translation.y);
            if test_x.distance(world_pos) < collision_distance {
                if is_cracked {
                    continue;
                }
                blocked_x = true;
                if !eggs_to_crack.contains(&entity) {
                    eggs_to_crack.push(entity);
                }
            }

            let test_y = Vec2::new(transform.translation.x, new_pos.y);
            if test_y.distance(world_pos) < collision_distance {
                if is_cracked {
                    continue;
                }
                blocked_y = true;
                if !eggs_to_crack.contains(&entity) {
                    eggs_to_crack.push(entity);
                }
            }
        }

        for entity in eggs_to_crack {
            commands.entity(entity).insert((
                Cracked,
                MeshMaterial2d(egg_materials.cracked.clone()),
            ));

            let mut rng = rand::rng();
            match rng.random_range(0..3) {
                0 => stats.philosophy += 1,
                1 => stats.nature_study += 1,
                _ => stats.wisdom += 1,
            }
        }

        if !blocked_x {
            transform.translation.x = new_pos.x;
        }
        if !blocked_y {
            transform.translation.y = new_pos.y;
        }
    }
}

fn camera_follow(
    egg_query: Query<&Transform, With<Egg>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Egg>)>,
) {
    let Ok(egg_transform) = egg_query.get_single() else { return };
    let Ok(mut camera_transform) = camera_query.get_single_mut() else { return };

    camera_transform.translation.x = egg_transform.translation.x;
    camera_transform.translation.y = egg_transform.translation.y;
}

fn update_counters(
    stats: Res<Stats>,
    mut philosophy_query: Query<&mut Text, (With<PhilosophyCounter>, Without<NatureStudyCounter>, Without<WisdomCounter>)>,
    mut nature_query: Query<&mut Text, (With<NatureStudyCounter>, Without<PhilosophyCounter>, Without<WisdomCounter>)>,
    mut wisdom_query: Query<&mut Text, (With<WisdomCounter>, Without<PhilosophyCounter>, Without<NatureStudyCounter>)>,
) {
    if let Ok(mut text) = philosophy_query.get_single_mut() {
        **text = stats.philosophy.to_string();
    }
    if let Ok(mut text) = nature_query.get_single_mut() {
        **text = stats.nature_study.to_string();
    }
    if let Ok(mut text) = wisdom_query.get_single_mut() {
        **text = stats.wisdom.to_string();
    }
}
