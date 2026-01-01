use clap::{Parser, Subcommand};
use image::{Rgba, RgbaImage};
use rand::Rng;
use std::fs;

#[derive(Parser)]
#[command(name = "pixel-gen")]
#[command(about = "Generate pixel art sprites and textures")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate an autotile set for terrain transitions
    Autotile {
        /// Primary terrain type: grass, dirt, stone, water, sand
        #[arg(short, long, default_value = "water")]
        terrain: String,

        /// Border/transition terrain type
        #[arg(short, long, default_value = "grass")]
        border: String,

        /// Tile size in pixels
        #[arg(short, long, default_value = "16")]
        size: u32,

        /// Output file path
        #[arg(short, long, default_value = "autotile.png")]
        output: String,

        /// Generate a preview terrain map instead of tileset
        #[arg(short, long, default_value = "false")]
        preview: bool,
    },

    /// Generate a sprite from parameters or spec file
    Sprite {
        /// Load spec from JSON file
        #[arg(long)]
        spec: Option<String>,

        /// Save spec to JSON file
        #[arg(long)]
        save_spec: Option<String>,

        /// Sprite width in pixels
        #[arg(short = 'W', long, default_value = "16")]
        width: u32,

        /// Sprite height in pixels
        #[arg(short = 'H', long, default_value = "16")]
        height: u32,

        /// Shape: circle, blob, square
        #[arg(short, long, default_value = "blob")]
        shape: String,

        /// Base color as hex (e.g., ff4444)
        #[arg(short, long, default_value = "ff4444")]
        color: String,

        /// Output file path
        #[arg(short, long, default_value = "sprite.png")]
        output: String,

        /// Enable horizontal symmetry
        #[arg(long, default_value = "true")]
        symmetry: bool,

        /// Number of eyes (0-2)
        #[arg(long, default_value = "0")]
        eyes: u8,

        /// Eye style: dot, round, angry
        #[arg(long, default_value = "round")]
        eye_style: String,

        /// Mouth style: none, happy, sad, open
        #[arg(long, default_value = "none")]
        mouth: String,
    },

    /// Generate a terrain texture
    Terrain {
        /// Load spec from JSON file
        #[arg(long)]
        spec: Option<String>,

        /// Save spec to JSON file
        #[arg(long)]
        save_spec: Option<String>,

        /// Texture width in pixels
        #[arg(short = 'W', long, default_value = "32")]
        width: u32,

        /// Texture height in pixels
        #[arg(short = 'H', long, default_value = "32")]
        height: u32,

        /// Terrain type: grass, dirt, stone, water, sand
        #[arg(short, long, default_value = "grass")]
        terrain: String,

        /// Base color as hex (overrides terrain default)
        #[arg(short, long)]
        color: Option<String>,

        /// Output file path
        #[arg(short, long, default_value = "terrain.png")]
        output: String,

        /// Noise density (0.0-1.0)
        #[arg(long, default_value = "0.3")]
        noise: f32,

        /// Enable tiling (seamless edges)
        #[arg(long, default_value = "true")]
        tile: bool,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SpriteSpec {
    width: u32,
    height: u32,
    shape: String,
    color: String,
    symmetry: bool,
    eyes: u8,
    eye_style: String,
    mouth: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct TerrainSpec {
    width: u32,
    height: u32,
    terrain: String,
    color: Option<String>,
    noise: f32,
    tile: bool,
}

fn parse_hex_color(hex: &str) -> Rgba<u8> {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(100);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(100);
    Rgba([r, g, b, 255])
}

fn darken(color: Rgba<u8>, amount: f32) -> Rgba<u8> {
    Rgba([
        (color[0] as f32 * (1.0 - amount)) as u8,
        (color[1] as f32 * (1.0 - amount)) as u8,
        (color[2] as f32 * (1.0 - amount)) as u8,
        color[3],
    ])
}

fn lighten(color: Rgba<u8>, amount: f32) -> Rgba<u8> {
    Rgba([
        (color[0] as f32 + (255.0 - color[0] as f32) * amount) as u8,
        (color[1] as f32 + (255.0 - color[1] as f32) * amount) as u8,
        (color[2] as f32 + (255.0 - color[2] as f32) * amount) as u8,
        color[3],
    ])
}

fn generate_blob_mask(width: u32, height: u32, symmetry: bool) -> Vec<Vec<bool>> {
    let mut rng = rand::rng();
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let rx = width as f32 / 2.5;
    let ry = height as f32 / 2.2;

    let half_width = if symmetry { (width + 1) / 2 } else { width };
    let mut mask = vec![vec![false; height as usize]; width as usize];

    for x in 0..half_width {
        for y in 0..height {
            let dx = (x as f32 - cx) / rx;
            let dy = (y as f32 - cy) / ry;
            let dist = (dx * dx + dy * dy).sqrt();
            let threshold = 1.0 + rng.random_range(-0.2..0.2);
            if dist < threshold {
                mask[x as usize][y as usize] = true;
                if symmetry {
                    mask[(width - 1 - x) as usize][y as usize] = true;
                }
            }
        }
    }
    mask
}

fn generate_circle_mask(width: u32, height: u32) -> Vec<Vec<bool>> {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let r = (width.min(height) as f32 / 2.0) - 1.0;

    let mut mask = vec![vec![false; height as usize]; width as usize];
    for x in 0..width {
        for y in 0..height {
            let dx = x as f32 - cx + 0.5;
            let dy = y as f32 - cy + 0.5;
            if (dx * dx + dy * dy).sqrt() <= r {
                mask[x as usize][y as usize] = true;
            }
        }
    }
    mask
}

fn generate_square_mask(width: u32, height: u32) -> Vec<Vec<bool>> {
    let margin = 2u32;
    let mut mask = vec![vec![false; height as usize]; width as usize];
    for x in margin..(width - margin) {
        for y in margin..(height - margin) {
            mask[x as usize][y as usize] = true;
        }
    }
    mask
}

fn draw_eyes(img: &mut RgbaImage, mask: &[Vec<bool>], eyes: u8, style: &str) {
    if eyes == 0 {
        return;
    }

    let width = img.width();
    let height = img.height();
    let cx = width / 2;
    let eye_y = height / 3;

    let eye_positions: Vec<(i32, i32)> = match eyes {
        1 => vec![(cx as i32, eye_y as i32)],
        _ => vec![(cx as i32 - 3, eye_y as i32), (cx as i32 + 2, eye_y as i32)],
    };

    let white = Rgba([255, 255, 255, 255]);
    let black = Rgba([0, 0, 0, 255]);

    for (ex, ey) in eye_positions {
        match style {
            "dot" => {
                if ex >= 0 && ex < width as i32 && ey >= 0 && ey < height as i32 {
                    img.put_pixel(ex as u32, ey as u32, black);
                }
            }
            "angry" => {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let px = ex + dx;
                        let py = ey + dy;
                        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                            if mask[px as usize][py as usize] {
                                img.put_pixel(px as u32, py as u32, white);
                            }
                        }
                    }
                }
                img.put_pixel(ex as u32, ey as u32, black);
                // Angry brow
                if ey > 0 {
                    let brow_x = if ex < cx as i32 { ex + 1 } else { ex - 1 };
                    if brow_x >= 0 && brow_x < width as i32 {
                        img.put_pixel(brow_x as u32, (ey - 1) as u32, black);
                    }
                }
            }
            "round" | _ => {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let px = ex + dx;
                        let py = ey + dy;
                        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                            if mask[px as usize][py as usize] {
                                img.put_pixel(px as u32, py as u32, white);
                            }
                        }
                    }
                }
                img.put_pixel(ex as u32, ey as u32, black);
            }
        }
    }
}

fn draw_mouth(img: &mut RgbaImage, mask: &[Vec<bool>], mouth: &str) {
    if mouth == "none" {
        return;
    }

    let width = img.width();
    let height = img.height();
    let cx = width as i32 / 2;
    let mouth_y = (height * 2 / 3) as i32;
    let black = Rgba([20, 20, 20, 255]);
    let tongue = Rgba([255, 100, 100, 255]);

    match mouth {
        "happy" => {
            let offsets = [(-2, -1), (-1, 0), (0, 0), (1, 0), (2, -1)];
            for (dx, dy) in offsets {
                let px = cx + dx;
                let py = mouth_y + dy;
                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    if mask[px as usize][py as usize] {
                        img.put_pixel(px as u32, py as u32, black);
                    }
                }
            }
        }
        "sad" => {
            let offsets = [(-2, 1), (-1, 0), (0, 0), (1, 0), (2, 1)];
            for (dx, dy) in offsets {
                let px = cx + dx;
                let py = mouth_y + dy;
                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    if mask[px as usize][py as usize] {
                        img.put_pixel(px as u32, py as u32, black);
                    }
                }
            }
        }
        "open" => {
            for dx in -2..=2 {
                for dy in 0..=2 {
                    let px = cx + dx;
                    let py = mouth_y + dy;
                    if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                        if mask[px as usize][py as usize] {
                            if dy == 2 && dx.abs() <= 1 {
                                img.put_pixel(px as u32, py as u32, tongue);
                            } else {
                                img.put_pixel(px as u32, py as u32, black);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn add_outline(img: &mut RgbaImage, outline_color: Rgba<u8>) {
    let width = img.width();
    let height = img.height();
    let mut outline_pixels = Vec::new();

    for x in 0..width {
        for y in 0..height {
            let pixel = img.get_pixel(x, y);
            if pixel[3] == 0 {
                let has_neighbor = [(-1, 0), (1, 0), (0, -1), (0, 1)]
                    .iter()
                    .any(|(dx, dy)| {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            img.get_pixel(nx as u32, ny as u32)[3] > 0
                        } else {
                            false
                        }
                    });
                if has_neighbor {
                    outline_pixels.push((x, y));
                }
            }
        }
    }

    for (x, y) in outline_pixels {
        img.put_pixel(x, y, outline_color);
    }
}

fn generate_sprite(spec: &SpriteSpec) -> RgbaImage {
    let mut img = RgbaImage::new(spec.width, spec.height);
    let base_color = parse_hex_color(&spec.color);

    let mask = match spec.shape.as_str() {
        "circle" => generate_circle_mask(spec.width, spec.height),
        "square" => generate_square_mask(spec.width, spec.height),
        _ => generate_blob_mask(spec.width, spec.height, spec.symmetry),
    };

    // Fill with shaded color
    for x in 0..spec.width {
        for y in 0..spec.height {
            if mask[x as usize][y as usize] {
                let shade = (y as f32 / spec.height as f32 - 0.3).clamp(0.0, 0.4);
                let highlight = y < spec.height / 3
                    && (x as i32 - spec.width as i32 / 2).abs() < (spec.width / 4) as i32;

                let color = if highlight {
                    lighten(base_color, 0.3)
                } else {
                    darken(base_color, shade)
                };
                img.put_pixel(x, y, color);
            }
        }
    }

    draw_eyes(&mut img, &mask, spec.eyes, &spec.eye_style);
    draw_mouth(&mut img, &mask, &spec.mouth);
    add_outline(&mut img, darken(base_color, 0.5));

    img
}

fn get_terrain_base_color(terrain: &str) -> Rgba<u8> {
    match terrain {
        "grass" => parse_hex_color("4a8c4a"),
        "dirt" => parse_hex_color("8b6b4a"),
        "stone" => parse_hex_color("7a7a7a"),
        "water" => parse_hex_color("4a7ab8"),
        "sand" => parse_hex_color("d4b86a"),
        _ => parse_hex_color("4a8c4a"),
    }
}

fn generate_terrain(spec: &TerrainSpec) -> RgbaImage {
    let mut img = RgbaImage::new(spec.width, spec.height);
    let mut rng = rand::rng();

    let base_color = spec
        .color
        .as_ref()
        .map(|c| parse_hex_color(c))
        .unwrap_or_else(|| get_terrain_base_color(&spec.terrain));

    // Fill base
    for x in 0..spec.width {
        for y in 0..spec.height {
            let variation = rng.random_range(-0.1..0.1);
            let color = if variation > 0.0 {
                lighten(base_color, variation)
            } else {
                darken(base_color, -variation)
            };
            img.put_pixel(x, y, color);
        }
    }

    // Add noise/detail
    let detail_count = ((spec.width * spec.height) as f32 * spec.noise) as u32;
    for _ in 0..detail_count {
        let x = rng.random_range(0..spec.width);
        let y = rng.random_range(0..spec.height);
        let variation = rng.random_range(0.1..0.3);
        let darker = rng.random_bool(0.5);
        let color = if darker {
            darken(base_color, variation)
        } else {
            lighten(base_color, variation)
        };
        img.put_pixel(x, y, color);
    }

    // Add terrain-specific details
    match spec.terrain.as_str() {
        "grass" => {
            let blade_count = (spec.width * spec.height / 8) as u32;
            let blade_color = lighten(base_color, 0.2);
            for _ in 0..blade_count {
                let x = rng.random_range(0..spec.width);
                let y = rng.random_range(0..spec.height);
                img.put_pixel(x, y, blade_color);
                if y > 0 {
                    img.put_pixel(x, y - 1, lighten(blade_color, 0.1));
                }
            }
        }
        "water" => {
            let wave_color = lighten(base_color, 0.4);
            for y in (0..spec.height).step_by(4) {
                let offset = rng.random_range(0..3) as i32;
                for x in 0..spec.width {
                    let wave_x = ((x as f32 * 0.8).sin() * 1.5) as i32;
                    let py = (y as i32 + wave_x + offset).clamp(0, spec.height as i32 - 1) as u32;
                    img.put_pixel(x, py, wave_color);
                    if py + 1 < spec.height {
                        img.put_pixel(x, py + 1, lighten(base_color, 0.2));
                    }
                }
            }
        }
        "stone" => {
            let crack_count = spec.width / 6;
            let crack_color = darken(base_color, 0.3);
            for _ in 0..crack_count {
                let mut x = rng.random_range(0..spec.width) as i32;
                let mut y = rng.random_range(0..spec.height) as i32;
                for _ in 0..rng.random_range(3..8) {
                    if x >= 0 && x < spec.width as i32 && y >= 0 && y < spec.height as i32 {
                        img.put_pixel(x as u32, y as u32, crack_color);
                    }
                    x += rng.random_range(-1..=1);
                    y += rng.random_range(0..=1);
                }
            }
        }
        _ => {}
    }

    img
}

fn generate_autotile_preview(terrain: &str, border: &str, size: u32) -> RgbaImage {
    let tileset = generate_autotile(terrain, border, size);

    // Create a sample terrain: water blob in center of grass
    let map_w = 8;
    let map_h = 6;
    let terrain_map: [[bool; 8]; 6] = [
        [false, false, false, false, false, false, false, false],
        [false, false, true,  true,  true,  true,  false, false],
        [false, true,  true,  true,  true,  true,  true,  false],
        [false, true,  true,  true,  true,  true,  true,  false],
        [false, false, true,  true,  true,  false, false, false],
        [false, false, false, false, false, false, false, false],
    ];

    let mut img = RgbaImage::new(map_w as u32 * size, map_h as u32 * size);

    for my in 0..map_h {
        for mx in 0..map_w {
            // Check corners for this tile by looking at 2x2 neighbors
            let get = |x: i32, y: i32| -> bool {
                if x < 0 || y < 0 || x >= map_w as i32 || y >= map_h as i32 {
                    false
                } else {
                    terrain_map[y as usize][x as usize]
                }
            };

            let x = mx as i32;
            let y = my as i32;

            // For blob autotile, check the 4 quadrants
            let nw = get(x, y) && get(x - 1, y) && get(x, y - 1) && get(x - 1, y - 1);
            let ne = get(x, y) && get(x + 1, y) && get(x, y - 1) && get(x + 1, y - 1);
            let sw = get(x, y) && get(x - 1, y) && get(x, y + 1) && get(x - 1, y + 1);
            let se = get(x, y) && get(x + 1, y) && get(x, y + 1) && get(x + 1, y + 1);

            let tile_idx = (nw as u32) | ((ne as u32) << 1) | ((sw as u32) << 2) | ((se as u32) << 3);
            let tile_x = (tile_idx % 4) * size;
            let tile_y = (tile_idx / 4) * size;

            // Copy tile to output
            for ty in 0..size {
                for tx in 0..size {
                    let pixel = tileset.get_pixel(tile_x + tx, tile_y + ty);
                    img.put_pixel(mx as u32 * size + tx, my as u32 * size + ty, *pixel);
                }
            }
        }
    }

    img
}

fn generate_autotile(terrain: &str, border: &str, size: u32) -> RgbaImage {
    let mut rng = rand::rng();
    let terrain_color = get_terrain_base_color(terrain);
    let border_color = get_terrain_base_color(border);
    let edge_color = darken(terrain_color, 0.3);

    // 4x4 grid = 16 tiles for blob autotile
    let img_size = size * 4;
    let mut img = RgbaImage::new(img_size, img_size);

    // Fill with border color first
    for x in 0..img_size {
        for y in 0..img_size {
            let variation = rng.random_range(-0.05..0.05);
            let color = if variation > 0.0 {
                lighten(border_color, variation)
            } else {
                darken(border_color, -variation)
            };
            img.put_pixel(x, y, color);
        }
    }

    // Tile index defines which corners have terrain (bit flags: NW=1, NE=2, SW=4, SE=8)
    for tile_idx in 0..16u32 {
        let tile_x = (tile_idx % 4) * size;
        let tile_y = (tile_idx / 4) * size;

        let nw = tile_idx & 1 != 0;
        let ne = tile_idx & 2 != 0;
        let sw = tile_idx & 4 != 0;
        let se = tile_idx & 8 != 0;

        let half = size / 2;

        // First pass: fill terrain with pattern
        for x in 0..size {
            for y in 0..size {
                let in_left = x < half;
                let in_top = y < half;

                let fill = match (in_left, in_top) {
                    (true, true) => nw,
                    (false, true) => ne,
                    (true, false) => sw,
                    (false, false) => se,
                };

                if fill {
                    let px = tile_x + x;
                    let py = tile_y + y;
                    let variation = rng.random_range(-0.05..0.05);
                    let mut color = if variation > 0.0 {
                        lighten(terrain_color, variation)
                    } else {
                        darken(terrain_color, -variation)
                    };

                    // Add terrain-specific patterns
                    if terrain == "water" {
                        let wave_y = (y % 4) == 0;
                        let wave_offset = ((x as f32 * 0.8).sin() * 1.5) as i32;
                        if wave_y || (y as i32 + wave_offset) % 4 == 0 {
                            color = lighten(terrain_color, 0.3);
                        }
                    } else if terrain == "grass" {
                        if rng.random_bool(0.1) {
                            color = lighten(terrain_color, 0.2);
                        }
                    }

                    img.put_pixel(px, py, color);
                }
            }
        }

        // Second pass: add edge where terrain meets border
        for x in 0..size {
            for y in 0..size {
                let in_left = x < half;
                let in_top = y < half;

                let fill = match (in_left, in_top) {
                    (true, true) => nw,
                    (false, true) => ne,
                    (true, false) => sw,
                    (false, false) => se,
                };

                if fill {
                    let px = tile_x + x;
                    let py = tile_y + y;

                    // Check if adjacent to non-filled area
                    let check_neighbor = |dx: i32, dy: i32| -> bool {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= size as i32 || ny >= size as i32 {
                            return false;
                        }
                        let n_left = (nx as u32) < half;
                        let n_top = (ny as u32) < half;
                        let n_fill = match (n_left, n_top) {
                            (true, true) => nw,
                            (false, true) => ne,
                            (true, false) => sw,
                            (false, false) => se,
                        };
                        !n_fill
                    };

                    if check_neighbor(-1, 0) || check_neighbor(1, 0) ||
                       check_neighbor(0, -1) || check_neighbor(0, 1) {
                        img.put_pixel(px, py, edge_color);
                    }
                }
            }
        }
    }

    img
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Autotile {
            terrain,
            border,
            size,
            output,
            preview,
        } => {
            let img = if preview {
                generate_autotile_preview(&terrain, &border, size)
            } else {
                generate_autotile(&terrain, &border, size)
            };
            img.save(&output).expect("Failed to save image");
            if preview {
                println!(
                    "Generated autotile preview: {} ({}x{})",
                    output,
                    img.width(),
                    img.height()
                );
            } else {
                println!(
                    "Generated autotile: {} ({}x{}, {} tiles)",
                    output,
                    size * 4,
                    size * 4,
                    16
                );
            }
        }

        Commands::Sprite {
            spec,
            save_spec,
            width,
            height,
            shape,
            color,
            output,
            symmetry,
            eyes,
            eye_style,
            mouth,
        } => {
            let sprite_spec = if let Some(spec_path) = spec {
                let content = fs::read_to_string(&spec_path).expect("Failed to read spec file");
                serde_json::from_str(&content).expect("Failed to parse spec JSON")
            } else {
                SpriteSpec {
                    width,
                    height,
                    shape,
                    color,
                    symmetry,
                    eyes,
                    eye_style,
                    mouth,
                }
            };

            if let Some(save_path) = save_spec {
                let json = serde_json::to_string_pretty(&sprite_spec).unwrap();
                fs::write(&save_path, json).expect("Failed to save spec");
                println!("Saved spec: {}", save_path);
            }

            let img = generate_sprite(&sprite_spec);
            img.save(&output).expect("Failed to save image");
            println!("Generated sprite: {} ({}x{})", output, sprite_spec.width, sprite_spec.height);
        }

        Commands::Terrain {
            spec,
            save_spec,
            width,
            height,
            terrain,
            color,
            output,
            noise,
            tile,
        } => {
            let terrain_spec = if let Some(spec_path) = spec {
                let content = fs::read_to_string(&spec_path).expect("Failed to read spec file");
                serde_json::from_str(&content).expect("Failed to parse spec JSON")
            } else {
                TerrainSpec {
                    width,
                    height,
                    terrain,
                    color,
                    noise,
                    tile,
                }
            };

            if let Some(save_path) = save_spec {
                let json = serde_json::to_string_pretty(&terrain_spec).unwrap();
                fs::write(&save_path, json).expect("Failed to save spec");
                println!("Saved spec: {}", save_path);
            }

            let img = generate_terrain(&terrain_spec);
            img.save(&output).expect("Failed to save image");
            println!("Generated terrain: {} ({}x{})", output, terrain_spec.width, terrain_spec.height);
        }
    }
}
