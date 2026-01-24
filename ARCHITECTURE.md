# Eggo Architecture

## Module Dependency Hierarchy

```
core/               ← Pure foundation (NO game module imports)
    ↓
world/              ← Game configuration (imports inventory for ItemId)
inventory/          ← Items and weapons (imports core)
    ↓
combat/             ← Combat systems (imports inventory, creatures, player)
creatures/          ← Creature AI (imports combat, inventory, player)
player/             ← Player logic (imports combat, inventory, effects)
    ↓
effects/            ← Visual effects (imports core, player)
ui/                 ← User interface (imports inventory, player, creatures)
debug/              ← Debug tools (imports everything)
```

## Domain-Driven Structure

```
src/
├── main.rs              # App entry, plugin registration, UI setup
├── constants.rs         # Game constants (z-layers, sizes, etc.)
│
├── core/                # Shared infrastructure (NO game imports)
│   ├── assets.rs        # CharacterAssets (meshes, materials)
│   ├── collisions.rs    # WalkCollider, HitCollider, ellipse math
│   ├── components.rs    # Health, Dead, Stunned, Knockback, Shadow, Loot
│   ├── depth.rs         # YSorted, depth sorting
│   ├── input.rs         # InputBindings, GameAction
│   ├── state.rs         # GameState enum only
│   └── systems.rs       # update_stun, update_despawn_timer
│
├── world/               # Game configuration
│   └── mod.rs           # WorldConfig, NewGameRequested
│
├── state_machine/       # Generic state machine infrastructure
│   ├── mod.rs           # StateMachinePlugin, StateMachineSet, register_state_type
│   ├── traits.rs        # StateType trait
│   ├── machine.rs       # StateMachine<S> component
│   ├── events.rs        # RequestTransition, StateEntered, StateExited
│   ├── systems.rs       # process_transitions, tick_state_time
│   └── attack.rs        # AttackPhase, AttackTimer
│
├── inventory/           # Items and weapons domain
│   ├── components.rs    # Inventory, GroundItem, Pickupable
│   ├── data.rs          # Rarity, ItemId, ItemCategory, ItemRegistry
│   ├── systems.rs       # Pickup, hotbar, inventory UI interaction
│   ├── weapons/         # Weapon definitions
│   │   ├── data.rs      # Weapon, AttackType, DamageType, OnHitEffect, weapon_catalog
│   │   └── components.rs # Fist, Knife, Stick, WeaponSwing, Drawn, PlayerWeapon
│   └── items/           # Non-weapon items
│       ├── data.rs      # ConsumableEffect, item_catalog
│       └── components.rs # Armor, Consumable
│
├── player/              # Player domain
│   ├── components.rs    # Player, PlayerAnimation, Dashing, Sprinting
│   ├── events.rs        # DashInputDetected, AttackInputDetected, MovementInputDetected
│   ├── spawner.rs       # spawn_player, spawn_ground_item, spawn_background_grid
│   ├── sprites.rs       # PlayerSpriteSheet, Aseprite loader
│   ├── state.rs         # PlayerState enum
│   ├── state_handlers.rs # Input detection, state entry/exit handlers
│   ├── stats.rs         # Stats resource
│   └── systems.rs       # Movement, dash, knockback, animation, camera_follow
│
├── creatures/           # Creature domain
│   ├── components.rs    # Creature, CreatureAnimation, Hostile, Glowing, PatrolOrigin, PatrolWander, AlertIndicator
│   ├── data.rs          # CreatureDefinition, SteeringConfig (incl. patrol_radius), creature_catalog
│   ├── events.rs        # PlayerInRange and other creature events
│   ├── state.rs         # CreatureState enum (Idle, Patrol, Alert, Chase, Attack, Cooldown, etc.)
│   ├── state_handlers.rs # State enter/exit handlers, detect_player_proximity, alert handlers
│   ├── steering.rs      # Context steering (ContextMap, patrol_interest, patrol_boundary_danger)
│   ├── spawner.rs       # spawn_creatures, spawn_creature_range_indicator
│   └── systems.rs       # Animation, death, collision push, update_alert_indicator
│
├── combat/              # Combat systems domain
│   ├── components.rs    # Equipment, WeaponRangeIndicator, PlayerRangeIndicator
│   ├── hit_detection.rs # HitCone, arc intersection
│   ├── mesh.rs          # create_weapon_arc
│   └── systems.rs       # Attack, block, damage, AI (patrol_ai, alert_ai, hostile_ai, hostile_attack)
│
├── props/               # World props domain
│   ├── components.rs    # Prop, Destructible, CrateSprite, BarrelSprite
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

- `CorePlugin` - depth sorting, status timers (no game logic)
- `StateMachinePlugin` - state transition processing, system set ordering
- `PlayerPlugin` - movement, combat, animation, camera follow
- `CreaturePlugin` - AI, attack, death, state handlers
- `EffectsPlugin` - particles, screen effects
- `InventoryPlugin` - item management, hotbar
- `UiPlugin` - HUD, menus

## Game States

- `Loading` - initial state
- `Playing` - gameplay active
- `Paused` - game paused, menu visible
- `Dead` - player died, death screen visible

## Event-Driven Architecture (Target Approach)

The codebase follows **Event-Driven Architecture (EDA)** with the **Publish-Subscribe pattern**.

### Core Principle: Separation of Concerns

Systems are split into **Detection** (sensing) and **Decision** (acting):

```
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│    Detection    │  event  │    Decision     │  event  │     Action      │
│    (sensing)    │ ──────► │    (logic)      │ ──────► │   (execution)   │
└─────────────────┘         └─────────────────┘         └─────────────────┘

Example - Creature Attack:
detect_player_proximity → PlayerInRange → hostile_attack → RequestTransition → Attack state
```

### Benefits

- **Decoupling**: Systems don't know about each other, only events
- **Reusability**: Multiple systems can react to the same event
- **Extensibility**: Add new reactions without modifying existing code
- **Testability**: Events can be mocked/injected for testing

### Event Categories

| Category | Events | Purpose |
|----------|--------|---------|
| Creature | `PlayerInRange` | Detection/sensing |
| State Machine | `RequestTransition`, `StateEntered`, `StateExited` | State changes |
| (future) | `DamageTaken`, `EntityDied` | Combat reactions |

### Pattern: Don't Do Two Things

```rust
// BAD: Detection + Decision in one system
fn hostile_attack() {
    if distance < range {           // detection
        request_attack();           // decision
    }
}

// GOOD: Separated
fn detect_player_proximity() {
    if distance < range {
        emit(PlayerInRange);        // detection only
    }
}

fn hostile_attack() {
    for event in player_in_range {
        request_attack();           // decision only
    }
}
```

---

## State Machine Architecture

Event-driven state machine where **states never change themselves directly**:

```
Behavior Systems → RequestTransition event → StateMachine validates → State changes
                                                      ↓
                                             StateEntered/StateExited events
                                                      ↓
                                Handler systems react (spawn/remove components)
```

### System Ordering (StateMachineSet)

1. `ProcessTransitions` - Validate and execute state changes, emit events
2. `OnEnter` - React to StateEntered (e.g., create WeaponSwing)
3. `OnExit` - React to StateExited (e.g., remove WeaponSwing)
4. `Behavior` - State-guarded logic (chase, attack check, phase advancement)
5. `Cleanup` - Animation completion, component removal

### CreatureState

```rust
enum CreatureState {
    Idle,                    // Neutral creatures, stationary
    Patrol,                  // Wandering near spawn point
    Alert,                   // Spotted player, showing "!" indicator
    Chase,                   // Pursuing player
    Attack(AttackPhase),     // WindUp → Strike → Recovery
    Cooldown,                // Post-attack recovery period
    Stunned,                 // Hit by player, temporarily disabled
    Dying,                   // Death animation playing
    Dead,                    // Corpse
}
```

### Creature State Flow

```
[Neutral spawn] → Idle
                    ↓ (hit by player, gets Hostile component)
[Hostile spawn] → Patrol ←─────────────────────────────────┐
                    ↓ (player in sight_range)              │
                  Alert (shows "!" for 1 sec)              │
                    ↓ (alert timer expires)                │
                    ↓ (player leaves) ─────────────────────┤
                  Chase ←──────────────────┐               │
                    ↓ (player in range)    │               │
              Attack(WindUp)               │               │
                    ↓ (timer >= hit_delay) │               │
              Attack(Strike)               │               │
                    ↓ (hit applied)        │               │
              Attack(Recovery)             │               │
                    ↓ (timer >= duration)  │               │
                 Cooldown                  │               │
                    ↓ (cooldown expires)   │               │
                    └──────────────────────┘               │
                    ↓ (player exceeds chase_range)         │
                    └──────────────────────────────────────┘
```

## Creature Behavior

- `Creature` = neutral blob, starts in `Idle` state
- `Creature` + `Hostile` = patrols and attacks, starts in `Patrol` state
- `Creature` + `Glowing` = special variant (visual only)
- When neutral creature is hit: gets `Hostile`, transitions `Idle → Chase`

### Patrol Behavior

Hostile creatures patrol near their spawn point (`PatrolOrigin`):
- Randomly alternate between moving and idling (`PatrolWander.action`)
- Stay within `patrol_radius` of spawn point
- When player enters `sight_range`: transition to `Alert` state

### Alert State

When a creature spots the player:
- Shows "!" indicator above head (`AlertIndicator`)
- Pauses for 1 second, facing player
- If player leaves sight range: return to `Patrol`
- If player stays: transition to `Chase` and pursue

### Context Steering

Hostile creatures use context-based steering with interest/danger maps:
- **Interest**: Direction toward player (direct or flanking), or wander direction (patrol)
- **Danger**: Obstacles, pits, other creatures, player proximity, patrol boundary

## Combat System

- Weapons defined in `inventory/weapons/` with `AttackType`: Slash, Stab, Smash
- Hit detection uses ellipse-arc intersection
- Blocking reduces damage and reflects knockback
- Hitstop freezes action on hit for game feel
- Creature attacks go through state machine phases (WindUp → Strike → Recovery → Cooldown)
- Knockback applies to both living creatures and corpses
- Death animation waits for knockback to complete before playing
