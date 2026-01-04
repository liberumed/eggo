# Eggo Architecture

## Domain-Driven Structure

```
src/
├── main.rs              # App entry, plugin registration, UI setup
├── constants.rs         # Game constants (z-layers, sizes, etc.)
│
├── core/                # Shared infrastructure
│   ├── assets.rs        # CharacterAssets (meshes, materials)
│   ├── camera.rs        # Camera follow system
│   ├── collisions.rs    # WalkCollider, HitCollider, ellipse math
│   ├── components.rs    # Health, Dead, Stunned, Knockback, Shadow, Loot
│   ├── depth.rs         # YSorted, depth sorting
│   ├── input.rs         # InputBindings, GameAction
│   ├── state.rs         # GameState, WorldConfig
│   └── systems.rs       # update_stun, update_despawn_timer
│
├── player/              # Player domain
│   ├── components.rs    # Player, PlayerAnimation, Dashing, Sprinting
│   ├── spawner.rs       # spawn_player, spawn_ground_item, spawn_background_grid
│   ├── sprites.rs       # PlayerSpriteSheet, Aseprite loader
│   ├── stats.rs         # Stats resource
│   └── systems.rs       # Movement, dash, knockback, animation
│
├── creatures/           # Creature domain
│   ├── data.rs          # CreatureDefinition, creature_catalog
│   ├── spawner.rs       # spawn_creatures, spawn_creature_range_indicator
│   └── systems.rs       # Animation, death, collision push
│
├── combat/              # Combat domain
│   ├── hit_detection.rs # HitCone, arc intersection
│   ├── mesh.rs          # create_weapon_arc
│   ├── systems.rs       # Attack, block, damage, AI
│   └── weapons.rs       # Weapon, AttackType, weapon_catalog
│
├── inventory/           # Inventory domain
│   ├── components.rs    # Inventory, ItemId, ItemRegistry, GroundItem
│   └── systems.rs       # Pickup, hotbar, inventory UI interaction
│
├── props/               # World props domain
│   ├── components.rs    # Prop, Destructible, CrateSprite
│   ├── data.rs          # PropRegistry, PropDefinition
│   └── spawner.rs       # spawn_world_props
│
├── effects/             # Visual effects domain
│   ├── components.rs    # BloodParticle, ResourceBall, HitHighlight
│   ├── game_feel.rs     # ScreenShake, Hitstop
│   └── systems.rs       # Particle animation, magnetization
│
├── ui/                  # UI domain
│   ├── components.rs    # GameMenu, HotbarSlot, WeaponInfoPanel
│   └── systems.rs       # HUD updates, menu handling
│
└── debug/               # Debug tools
    ├── config.rs        # DebugConfig
    └── systems.rs       # Collision visualization, range cones
```

## Plugins

Each domain has a plugin in its `mod.rs`:

- `CorePlugin` - camera, depth sorting, status timers
- `PlayerPlugin` - movement, combat, animation
- `CreaturePlugin` - AI, attack, death
- `EffectsPlugin` - particles, screen effects
- `InventoryPlugin` - item management, hotbar
- `UiPlugin` - HUD, menus

## Game States

- `Loading` - initial state
- `Playing` - gameplay active
- `Paused` - game paused, menu visible
- `Dead` - player died, death screen visible

## Creature Behavior

- `Creature` = neutral blob
- `Creature` + `Hostile` = chases and attacks player
- `Creature` + `Glowing` = special variant (visual only)

## Combat System

- Weapons have `AttackType`: Slash, Stab, Smash
- Hit detection uses ellipse-arc intersection
- Blocking reduces damage and reflects knockback
- Hitstop freezes action on hit for game feel
