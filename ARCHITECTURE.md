# Eggo Architecture

## Structure

```
src/
├── main.rs                 # App entry, UI setup, system registration
├── constants.rs            # Game constants (z-layers, speeds, durations)
│
├── components/
│   ├── mod.rs
│   ├── common.rs           # Health, Dead, Stunned, Knockback, DespawnTimer, Shadow, Loot
│   ├── player.rs           # Player, PlayerAnimation
│   ├── creature.rs         # Creature, CreatureAnimation, Hostile, Glowing
│   ├── weapons.rs          # Knife, KnifeSwing, Fist, FistSwing
│   ├── effects.rs          # BloodParticle, ResourceBall, MagnetizedBall, TargetOutline
│   └── ui.rs               # DeathScreen, NewGameButton, counters
│
├── resources/
│   ├── mod.rs
│   └── stats.rs            # Stats (philosophy, nature_study, wisdom)
│
├── systems/
│   ├── mod.rs
│   ├── movement.rs         # move_player, apply_knockback
│   ├── animation.rs        # animate_player, animate_creatures, animate_knife_swing, etc.
│   ├── combat.rs           # knife_attack, aim_knife, hostile_ai, hostile_attack
│   ├── effects.rs          # animate_resource_balls, animate_magnetized_balls, animate_blood
│   ├── camera.rs           # camera_follow
│   ├── status.rs           # update_stun, update_despawn_timer
│   └── ui.rs               # update_counters, update_hp_text, show_death_screen
│
└── spawners/
    ├── mod.rs
    └── character.rs        # CharacterAssets, spawn_player, spawn_creatures, spawn_background_grid
```

## Creature Behavior

- `Creature` alone = neutral (won't attack)
- `Creature` + `Hostile` = chases and attacks player
- `Creature` + `Glowing` = special variant (yellow)
- When player hits neutral creature → adds `Hostile` + spawns fist

## Future Enhancements

### Plugin Architecture
Extract system groups into Bevy plugins for better organization:
```rust
app.add_plugins((
    PlayerPlugin,
    CreaturePlugin,
    EffectsPlugin,
    UiPlugin,
));
```

### Game States
Add state machine for menu/playing/paused/dead:
```rust
#[derive(States, Default)]
pub enum GameState {
    #[default]
    Playing,
    Paused,
    Dead,
}

app.add_systems(Update, move_player.run_if(in_state(GameState::Playing)));
```

### Unified Weapon Swing
Combine `KnifeSwing` and `FistSwing` into generic `WeaponSwing`:
```rust
pub struct WeaponSwing {
    pub timer: f32,
    pub duration: f32,
    pub base_angle: f32,
}
```
