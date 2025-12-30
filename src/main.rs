use bevy::prelude::*;
use rand::Rng;

const EGG_SPEED: f32 = 300.0;
const AGGRESSIVE_EGG_SPEED: f32 = 120.0;
const AGGRESSIVE_SIGHT_RANGE: f32 = 400.0;
const FIST_ATTACK_RANGE: f32 = EGG_RADIUS * 1.8;
const KNOCKBACK_FORCE: f32 = 500.0;
const GRID_SIZE: i32 = 20;
const GRID_SPACING: f32 = 100.0;
const EGG_RADIUS: f32 = 55.0;
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

#[derive(Component)]
struct BloodSplatter;

#[derive(Component)]
struct KnifeSwing {
    timer: f32,
    base_angle: f32,
}

#[derive(Component)]
struct DeathAnimation {
    timer: f32,
    stage: u8,
    pos: Vec2,
}

#[derive(Resource, Default)]
struct Stats {
    philosophy: u32,
    nature_study: u32,
    wisdom: u32,
}

#[derive(Resource)]
struct EggMaterials {
    blood: Handle<ColorMaterial>,
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
            camera_follow,
            update_counters,
        ))
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
    let aggressive_material = materials.add(Color::srgb(0.9, 0.4, 0.4));
    let glowing_material = materials.add(Color::srgb(1.0, 0.95, 0.5));
    let blood_material = materials.add(Color::srgb(0.8, 0.1, 0.1));

    let shadow_mesh = meshes.add(Ellipse::new(45.0, 25.0));
    let shadow_material = materials.add(Color::srgba(0.0, 0.0, 0.0, 0.2));

    let fist_mesh = meshes.add(Circle::new(12.0));
    let fist_material = materials.add(Color::srgb(0.85, 0.75, 0.6));

    commands.insert_resource(EggMaterials {
        blood: blood_material,
    });

    let knife_blade = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 8.0),
        Vec2::new(0.0, -8.0),
        Vec2::new(35.0, 0.0),
    ));
    let knife_handle = meshes.add(Rectangle::new(15.0, 10.0));
    let blade_material = materials.add(Color::srgb(0.7, 0.7, 0.75));
    let handle_material = materials.add(Color::srgb(0.4, 0.25, 0.1));

    commands.spawn((
        Egg,
        EggAnimation::default(),
        Health(2),
        Mesh2d(egg_mesh.clone()),
        MeshMaterial2d(player_material),
        Transform::default(),
    )).with_children(|parent| {
        parent.spawn((
            EggShadow,
            Mesh2d(shadow_mesh.clone()),
            MeshMaterial2d(shadow_material.clone()),
            Transform::from_xyz(5.0, -45.0, -0.1),
        ));
        parent.spawn((
            Knife,
            Transform::from_xyz(0.0, 0.0, 0.1),
            Visibility::default(),
        )).with_children(|knife| {
            knife.spawn((
                Mesh2d(knife_handle),
                MeshMaterial2d(handle_material),
                Transform::from_xyz(45.0, 0.0, 0.0),
            ));
            knife.spawn((
                Mesh2d(knife_blade),
                MeshMaterial2d(blade_material),
                Transform::from_xyz(60.0, 0.0, 0.0),
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
        fist_mesh,
        fist_material,
    );

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
}

fn spawn_world_eggs(
    commands: &mut Commands,
    egg_mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    aggressive_material: Handle<ColorMaterial>,
    glowing_material: Handle<ColorMaterial>,
    shadow_mesh: Handle<Mesh>,
    shadow_material: Handle<ColorMaterial>,
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

            let mut entity_commands = commands.spawn((
                WorldEgg,
                anim,
                Health(2),
                Mesh2d(egg_mesh.clone()),
                MeshMaterial2d(egg_material),
                Transform::from_xyz(x, y, 0.0),
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
                    Transform::from_xyz(5.0, -45.0, -0.1),
                ));

                if is_aggressive {
                    parent.spawn((
                        Fist,
                        Transform::from_xyz(0.0, 0.0, 0.1),
                        Visibility::default(),
                    )).with_children(|fist_holder| {
                        fist_holder.spawn((
                            Mesh2d(fist_mesh.clone()),
                            MeshMaterial2d(fist_material.clone()),
                            Transform::from_xyz(45.0, 0.0, 0.0),
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
            ..default()
        })
        .with_children(|parent| {
            spawn_stat_circle(parent, Color::srgb(0.6, 0.3, 0.7), PhilosophyCounter);
            spawn_stat_circle(parent, Color::srgb(0.3, 0.7, 0.3), NatureStudyCounter);
            spawn_stat_circle(parent, Color::srgb(0.9, 0.8, 0.2), WisdomCounter);
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
                BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
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

fn knife_attack(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Egg>>,
    knife_query: Query<(Entity, &Transform), With<Knife>>,
    knife_swing_query: Query<&KnifeSwing>,
    mut world_eggs_query: Query<(Entity, &Transform, &mut Health), (With<WorldEgg>, Without<Dead>, Without<DeathAnimation>)>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    if knife_swing_query.iter().next().is_some() {
        return;
    }

    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    if let Ok((knife_entity, knife_transform)) = knife_query.single() {
        let (_, angle) = knife_transform.rotation.to_axis_angle();
        let base_angle = if knife_transform.rotation.z < 0.0 { -angle } else { angle };
        commands.entity(knife_entity).insert(KnifeSwing { timer: 0.0, base_angle });
    }

    for (entity, egg_transform, mut health) in &mut world_eggs_query {
        let egg_pos = Vec2::new(egg_transform.translation.x, egg_transform.translation.y);

        if player_pos.distance(egg_pos) < KNIFE_RANGE {
            health.0 -= 1;

            if health.0 <= 0 {
                commands.entity(entity).insert(DeathAnimation {
                    timer: 0.0,
                    stage: 0,
                    pos: egg_pos,
                });
            }
        }
    }
}

fn aim_knife(
    player_query: Query<&Transform, With<Egg>>,
    mut knife_query: Query<&mut Transform, (With<Knife>, Without<Egg>, Without<KnifeSwing>)>,
    world_eggs_query: Query<&Transform, (With<WorldEgg>, Without<Dead>, Without<Egg>, Without<Knife>)>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let Ok(mut knife_transform) = knife_query.single_mut() else { return };

    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    let closest = world_eggs_query
        .iter()
        .map(|t| Vec2::new(t.translation.x, t.translation.y))
        .min_by(|a, b| {
            a.distance(player_pos)
                .partial_cmp(&b.distance(player_pos))
                .unwrap()
        });

    if let Some(target_pos) = closest {
        let dir = target_pos - player_pos;
        let angle = dir.y.atan2(dir.x);
        knife_transform.rotation = Quat::from_rotation_z(angle);
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
        let duration = 0.2;

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
    egg_materials: Res<EggMaterials>,
    mut stats: ResMut<Stats>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut Transform, &mut DeathAnimation), With<WorldEgg>>,
) {
    for (entity, mut transform, mut death) in &mut query {
        death.timer += time.delta_secs();
        let t = death.timer;

        match death.stage {
            0 => {
                let shake = (t * 60.0).sin() * 0.15 * (1.0 - t * 2.0).max(0.0);
                transform.rotation = Quat::from_rotation_z(shake);

                let expand = 1.0 + t * 0.5;
                transform.scale = Vec3::new(expand, expand * 0.9, 1.0);

                if t > 0.4 {
                    death.stage = 1;
                    death.timer = 0.0;

                    commands.entity(entity).insert((
                        Dead,
                        MeshMaterial2d(egg_materials.blood.clone()),
                    ));

                    let mut rng = rand::rng();
                    let blood_mesh = meshes.add(Circle::new(25.0));
                    for i in 0..12 {
                        let angle = i as f32 * std::f32::consts::TAU / 12.0 + rng.random_range(-0.3..0.3);
                        let dist = rng.random_range(30.0..80.0);
                        let offset = Vec2::from_angle(angle) * dist;
                        let size = rng.random_range(0.5..1.5);
                        commands.spawn((
                            BloodSplatter,
                            Mesh2d(blood_mesh.clone()),
                            MeshMaterial2d(egg_materials.blood.clone()),
                            Transform::from_xyz(
                                death.pos.x + offset.x,
                                death.pos.y + offset.y,
                                -0.5,
                            ).with_scale(Vec3::splat(size)),
                        ));
                    }

                    let splat_mesh = meshes.add(Ellipse::new(50.0, 35.0));
                    commands.spawn((
                        BloodSplatter,
                        Mesh2d(splat_mesh),
                        MeshMaterial2d(egg_materials.blood.clone()),
                        Transform::from_xyz(death.pos.x, death.pos.y - 20.0, -0.6),
                    ));

                    match rng.random_range(0..3) {
                        0 => stats.philosophy += 1,
                        1 => stats.nature_study += 1,
                        _ => stats.wisdom += 1,
                    }
                }
            }
            1 => {
                let squish = 1.2 - t * 0.8;
                let squash_x = squish.max(0.6) * 1.3;
                let squash_y = (2.0 - squish).min(1.4) * 0.5;
                transform.scale = Vec3::new(squash_x, squash_y, 1.0);

                if t > 0.3 {
                    commands.entity(entity).remove::<DeathAnimation>();
                    transform.scale = Vec3::new(1.0, 0.4, 1.0);
                }
            }
            _ => {}
        }
    }
}

fn move_egg(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut EggAnimation), With<Egg>>,
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
        Query<(Entity, &mut Transform), (With<Aggressive>, Without<Dead>, Without<Egg>)>,
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
    mut player_query: Query<(Entity, &Transform, &mut Health), (With<Egg>, Without<WorldEgg>)>,
    aggressive_query: Query<(&Transform, &Children), (With<Aggressive>, Without<Dead>)>,
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
