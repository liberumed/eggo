# Bevy 0.17 UI Image Sizing

## Problem
`ImageNode` auto-inserts `ContentSize` component which calculates size based on image intrinsic dimensions, overriding `Node` width/height settings.

## Solutions

### 1. Use `ContentSize::fixed_size()` (forces exact size)
```rust
slot.spawn((
    ImageNode {
        image_mode: bevy::ui::widget::NodeImageMode::Stretch,
        ..default()
    },
    Node {
        width: Val::Px(40.0),
        height: Val::Px(40.0),
        ..default()
    },
    bevy::ui::ContentSize::fixed_size(Vec2::new(40.0, 40.0)),
));
```
Note: `Stretch` mode ignores aspect ratio - image will be distorted to fill.

### 2. Use `Auto` mode with max constraints (preserves aspect ratio)
```rust
slot.spawn((
    ImageNode::default(), // Auto mode is default
    Node {
        max_width: Val::Px(40.0),
        max_height: Val::Px(40.0),
        ..default()
    },
));
```
Note: Image keeps aspect ratio but may not fill the slot.

### 3. Pre-size images (recommended for pixel art)
Export icons at exact target size (e.g., 40x40 for inventory slots).
This avoids runtime scaling issues entirely.

## References
- https://docs.rs/bevy/latest/bevy/ui/struct.ContentSize.html
- https://github.com/bevyengine/bevy/issues/16109
- https://github.com/bevyengine/bevy/pull/16083
