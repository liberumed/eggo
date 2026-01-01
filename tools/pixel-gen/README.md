# pixel-gen

CLI tool for generating pixel art sprites and terrain textures.

## Build

```bash
cargo build --release
```

## Output

Generated files go in `samples/` (gitignored):

```bash
pixel-gen sprite -o samples/creature.png
```

## Usage

### Sprites

```bash
# Basic sprite
pixel-gen sprite -o samples/creature.png

# Customized sprite
pixel-gen sprite --color ff4444 --shape blob --eyes 2 --eye-style round --mouth happy -o samples/slime.png

# Save spec for iteration
pixel-gen sprite --color 44ff44 --eyes 2 --save-spec samples/spec.json -o samples/sprite.png

# Generate from spec
pixel-gen sprite --spec samples/spec.json -o samples/sprite_v2.png
```

**Sprite options:**
- `--color <hex>` - Base color (default: ff4444)
- `--shape <type>` - blob, circle, square (default: blob)
- `-W, --width <px>` - Width in pixels (default: 16)
- `-H, --height <px>` - Height in pixels (default: 16)
- `--symmetry <bool>` - Horizontal symmetry (default: true)
- `--eyes <0-2>` - Number of eyes (default: 0)
- `--eye-style <type>` - dot, round, angry (default: round)
- `--mouth <type>` - none, happy, sad, open (default: none)
- `--spec <file>` - Load from JSON spec
- `--save-spec <file>` - Save spec to JSON

### Terrain

```bash
# Basic terrain
pixel-gen terrain --terrain grass -o samples/grass.png

# Customized terrain
pixel-gen terrain --terrain water --noise 0.4 -W 64 -H 64 -o samples/water.png

# With spec
pixel-gen terrain --terrain stone --save-spec samples/stone.json -o samples/stone.png
```

**Terrain options:**
- `--terrain <type>` - grass, dirt, stone, water, sand (default: grass)
- `--color <hex>` - Override base color
- `-W, --width <px>` - Width in pixels (default: 32)
- `-H, --height <px>` - Height in pixels (default: 32)
- `--noise <0.0-1.0>` - Detail density (default: 0.3)
- `--tile <bool>` - Seamless tiling (default: true)
- `--spec <file>` - Load from JSON spec
- `--save-spec <file>` - Save spec to JSON

## Iteration Workflow

1. Generate with `--save-spec`:
   ```bash
   pixel-gen sprite --color ff8800 --eyes 2 --save-spec samples/creature.json -o samples/creature.png
   ```

2. Edit `samples/creature.json` to adjust parameters

3. Regenerate from spec:
   ```bash
   pixel-gen sprite --spec samples/creature.json -o samples/creature_v2.png
   ```

## Spec Format

**Sprite spec:**
```json
{
  "width": 16,
  "height": 16,
  "shape": "blob",
  "color": "ff4444",
  "symmetry": true,
  "eyes": 2,
  "eye_style": "round",
  "mouth": "happy"
}
```

**Terrain spec:**
```json
{
  "width": 32,
  "height": 32,
  "terrain": "grass",
  "color": null,
  "noise": 0.3,
  "tile": true
}
```
