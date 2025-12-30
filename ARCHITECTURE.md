# Eggo Architecture

```
src/
├── main.rs              # App entry, UI setup
├── constants.rs         # Game constants
├── components/          # ECS components
├── resources/           # GameState, Stats
├── systems/             # Game logic
├── plugins/             # System groupings
└── spawners/            # Entity creation
```

## Plugins

- `PlayerPlugin` - movement, combat, animation
- `CreaturePlugin` - AI, death
- `EffectsPlugin` - particles, blood
- `UiPlugin` - HUD, death screen
- `StatusPlugin` - camera, timers

## Game States

- `Playing` - gameplay active
- `Dead` - frozen, death screen visible

## Creature Behavior

- `Creature` = neutral
- `Creature` + `Hostile` = attacks player
- `Creature` + `Glowing` = special variant
