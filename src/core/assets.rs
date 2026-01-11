use bevy::prelude::*;

#[derive(Resource)]
pub struct CharacterAssets {
    // Base character mesh
    pub character_mesh: Handle<Mesh>,

    // Character materials
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

    // Outline
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<ColorMaterial>,

    // Blood effects
    pub blood_splat_mesh: Handle<Mesh>,
    pub blood_droplet_mesh: Handle<Mesh>,
    pub blood_splat_material: Handle<ColorMaterial>,
    pub blood_droplet_material: Handle<ColorMaterial>,

    // Ground items
    pub item_glow_mesh: Handle<Mesh>,
    pub item_glow_material: Handle<ColorMaterial>,

    // Weapon range indicator (mesh created dynamically per-weapon)
    pub range_indicator_material: Handle<ColorMaterial>,
    pub attack_windup_material: Handle<ColorMaterial>,
    pub attack_strike_material: Handle<ColorMaterial>,
}

impl CharacterAssets {
    pub fn load(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Self {
        let character_mesh = meshes.add(Ellipse::new(10.0, 14.0));

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

        let outline_mesh = meshes.add(Ellipse::new(11.7, 16.25));
        let outline_material = materials.add(Color::srgba(0.75, 1.0, 0.0, 0.9));

        let blood_splat_mesh = meshes.add(Ellipse::new(4.0, 3.0));
        let blood_droplet_mesh = meshes.add(Circle::new(2.0));
        let blood_splat_material = materials.add(Color::srgb(0.7, 0.0, 0.0));
        let blood_droplet_material = materials.add(Color::srgb(0.9, 0.1, 0.1));

        let item_glow_mesh = meshes.add(Circle::new(12.0));
        let item_glow_material = materials.add(Color::srgba(1.0, 1.0, 0.8, 0.3));

        let range_indicator_material = materials.add(Color::srgba(1.0, 0.2, 0.2, 0.8));
        let attack_windup_material = materials.add(Color::srgba(1.0, 0.5, 0.0, 0.5)); // Orange, semi-transparent
        let attack_strike_material = materials.add(Color::srgba(1.0, 0.1, 0.1, 0.9)); // Bright red

        Self {
            character_mesh,
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
            outline_mesh,
            outline_material,
            blood_splat_mesh,
            blood_droplet_mesh,
            blood_splat_material,
            blood_droplet_material,
            item_glow_mesh,
            item_glow_material,
            range_indicator_material,
            attack_windup_material,
            attack_strike_material,
        }
    }
}
