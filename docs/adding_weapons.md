# Adding a New Weapon

## Overview
Adding a new weapon requires changes in 4-5 files. Follow these steps:

## Step 1: Add ItemId Variant

**File: `src/components/inventory.rs`**

Add new variant to `ItemId` enum:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemId {
    WoodenStick,
    RustyKnife,
    Fist,
    NewWeapon,  // Add here
    // ...
}
```

## Step 2: Create Weapon Definition

**File: `src/components/weapons.rs`**

Add function in `weapon_catalog` module:
```rust
pub mod weapon_catalog {
    pub fn new_weapon(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Weapon {
        Weapon {
            name: "New Weapon".to_string(),
            visual: WeaponVisual {
                mesh: meshes.add(Rectangle::new(15.0, 3.0)),  // Shape
                material: materials.add(Color::srgb(0.8, 0.2, 0.2)),  // Color
                offset: 12.0,  // Distance from player center
            },
            damage: 2,       // 1-5 typical
            speed: 3,        // 1-5 (higher = faster attack)
            reach: 2,        // 1-5 (range in tiles)
            arc: 2,          // 1-5 (attack cone width)
            attack_type: AttackType::Slash,  // Slash, Smash, or Stab
            damage_type: DamageType::Physical,
            rarity: Rarity::Common,  // Common, Uncommon, Rare, Epic, Legendary
            cost: 15,
            block: 2,        // Block effectiveness
            block_kb: 2,     // Knockback resistance when blocking
            on_hit: vec![
                OnHitEffect::Stun { duration: 0.5 },
                OnHitEffect::Knockback { force: 150.0 },
            ],
        }
    }
}
```

### Weapon Stats Guide
| Stat | Range | Description |
|------|-------|-------------|
| damage | 1-5 | Base damage dealt |
| speed | 1-5 | Attack speed (5 = fastest) |
| reach | 1-5 | Attack range |
| arc | 1-5 | Width of attack cone |
| block | 1-5 | Damage reduction when blocking |
| block_kb | 1-5 | Knockback reduction when blocking |

### Attack Types
- `Slash` - Wide horizontal swing
- `Smash` - Overhead vertical swing
- `Stab` - Forward thrust

### On-Hit Effects
- `OnHitEffect::Stun { duration: f32 }` - Stun in seconds
- `OnHitEffect::Knockback { force: f32 }` - Push force (50-300 typical)

## Step 3: Add to Item Data

**File: `src/components/inventory.rs`**

### In `get_item_data()`:
```rust
pub fn get_item_data(id: ItemId) -> ItemData {
    match id {
        // ... existing weapons ...
        ItemId::NewWeapon => ItemData {
            id,
            name: "New Weapon".to_string(),
            category: ItemCategory::Weapon,
            stack_max: 1,
        },
    }
}
```

### In `get_weapon_stats()`:
```rust
pub fn get_weapon_stats(...) -> Option<super::Weapon> {
    match id {
        // ... existing weapons ...
        ItemId::NewWeapon => Some(weapon_catalog::new_weapon(meshes, materials)),
        _ => None,
    }
}
```

### In `build_item_registry()`:
```rust
// New Weapon
let new_weapon = weapon_catalog::new_weapon(meshes, materials);
items.insert(
    ItemId::NewWeapon,
    ItemDefinition {
        name: new_weapon.name.clone(),
        category: ItemCategory::Weapon,
        stack_max: 1,
        ground_visual: GroundItemVisual {
            meshes: vec![(
                new_weapon.visual.mesh.clone(),
                new_weapon.visual.material.clone(),
                Vec3::ZERO,
            )],
        },
        weapon: Some(new_weapon),
        consumable_effect: None,
    },
);
```

## Step 4: Add Icons (Optional but Recommended)

### Export from Aseprite
```bash
# UI icon (for hotbar/inventory) - scale 6x, trimmed
aseprite -b weapon.aseprite --trim --scale 6 --save-as assets/sprites/items/weapon.png

# Ground sprite (for dropped items) - scale 3x, trimmed
aseprite -b weapon.aseprite --trim --scale 3 --save-as assets/sprites/items/weapon_ground.png
```

### Register Icons

**File: `src/main.rs`** in `setup_game()`:
```rust
// Load item icons
item_icons.icons.insert(
    components::ItemId::NewWeapon,
    asset_server.load("sprites/items/weapon.png")
);
item_icons.ground_icons.insert(
    components::ItemId::NewWeapon,
    asset_server.load("sprites/items/weapon_ground.png")
);
```

## Step 5: Add to World (Optional)

To spawn weapon on ground at game start:

**File: `src/resources/world.rs`** in `WorldConfig`:
```rust
starting_items: vec![
    (ItemId::NewWeapon, 1, Vec2::new(100.0, 50.0)),
],
```

Or give to player in inventory:

**File: `src/spawners/character.rs`** in `spawn_player()`:
```rust
inventory.try_add(ItemId::NewWeapon, 1);
```

## File Summary

| File | Changes |
|------|---------|
| `src/components/inventory.rs` | ItemId enum, get_item_data, get_weapon_stats, build_item_registry |
| `src/components/weapons.rs` | weapon_catalog function |
| `src/main.rs` | Icon loading (optional) |
| `assets/sprites/items/` | Icon files (optional) |
