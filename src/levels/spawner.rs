use bevy::prelude::*;

use super::LevelData;

#[derive(Component)]
pub struct LevelBackground;

#[derive(Component)]
pub struct VoidBackground;

const Z_VOID: f32 = -10.0;
const Z_CORRIDOR: f32 = -9.0;

pub fn spawn_level_background(
    commands: &mut Commands,
    level: &LevelData,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let bounds = &level.bounds;

    let void_color = Color::srgb(0.05, 0.05, 0.08);
    let corridor_color = Color::srgb(0.15, 0.15, 0.18);

    let void_size = Vec2::new(2000.0, 2000.0);
    let void_mesh = meshes.add(Rectangle::new(void_size.x, void_size.y));
    let void_material = materials.add(void_color);

    commands.spawn((
        VoidBackground,
        Mesh2d(void_mesh),
        MeshMaterial2d(void_material),
        Transform::from_xyz(bounds.center().x, bounds.center().y, Z_VOID),
    ));

    for walkable in &level.walkable {
        let width = walkable.max.x - walkable.min.x;
        let height = walkable.max.y - walkable.min.y;
        let center_x = (walkable.min.x + walkable.max.x) / 2.0;
        let center_y = (walkable.min.y + walkable.max.y) / 2.0;

        let corridor_mesh = meshes.add(Rectangle::new(width, height));
        let corridor_material = materials.add(corridor_color);

        commands.spawn((
            LevelBackground,
            Mesh2d(corridor_mesh),
            MeshMaterial2d(corridor_material),
            Transform::from_xyz(center_x, center_y, Z_CORRIDOR),
        ));
    }
}
