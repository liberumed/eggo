use bevy::prelude::*;
use std::collections::HashMap;

/// All game actions that can be bound to input
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameAction {
    // Movement
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Sprint,
    Dash,

    // Combat
    Attack,
    Block,
    ToggleWeapon,

    // Inventory
    ToggleInventory,
    Hotbar1,
    Hotbar2,
    Hotbar3,
    Hotbar4,
    Hotbar5,
    InventoryUse,
    InventoryPickup,

    // UI
    Pause,
}

/// Input binding - either keyboard or mouse
#[derive(Clone, Copy, Debug)]
pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
}

/// Central resource for all input bindings
#[derive(Resource)]
pub struct InputBindings {
    bindings: HashMap<GameAction, InputBinding>,
}

impl Default for InputBindings {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        // Movement
        bindings.insert(GameAction::MoveUp, InputBinding::Key(KeyCode::KeyW));
        bindings.insert(GameAction::MoveDown, InputBinding::Key(KeyCode::KeyS));
        bindings.insert(GameAction::MoveLeft, InputBinding::Key(KeyCode::KeyA));
        bindings.insert(GameAction::MoveRight, InputBinding::Key(KeyCode::KeyD));
        bindings.insert(GameAction::Sprint, InputBinding::Key(KeyCode::ShiftLeft));
        bindings.insert(GameAction::Dash, InputBinding::Key(KeyCode::Space));

        // Combat
        bindings.insert(GameAction::Attack, InputBinding::Mouse(MouseButton::Left));
        bindings.insert(GameAction::Block, InputBinding::Mouse(MouseButton::Right));
        bindings.insert(GameAction::ToggleWeapon, InputBinding::Key(KeyCode::KeyR));

        // Inventory
        bindings.insert(GameAction::ToggleInventory, InputBinding::Key(KeyCode::Tab));
        bindings.insert(GameAction::Hotbar1, InputBinding::Key(KeyCode::Digit1));
        bindings.insert(GameAction::Hotbar2, InputBinding::Key(KeyCode::Digit2));
        bindings.insert(GameAction::Hotbar3, InputBinding::Key(KeyCode::Digit3));
        bindings.insert(GameAction::Hotbar4, InputBinding::Key(KeyCode::Digit4));
        bindings.insert(GameAction::Hotbar5, InputBinding::Key(KeyCode::Digit5));
        bindings.insert(GameAction::InventoryUse, InputBinding::Mouse(MouseButton::Right));
        bindings.insert(GameAction::InventoryPickup, InputBinding::Mouse(MouseButton::Left));

        // UI
        bindings.insert(GameAction::Pause, InputBinding::Key(KeyCode::Escape));

        Self { bindings }
    }
}

impl InputBindings {
    /// Check if action is currently held down
    pub fn pressed(
        &self,
        action: GameAction,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self.bindings.get(&action) {
            Some(InputBinding::Key(key)) => keyboard.pressed(*key),
            Some(InputBinding::Mouse(btn)) => mouse.pressed(*btn),
            None => false,
        }
    }

    /// Check if action was just pressed this frame
    pub fn just_pressed(
        &self,
        action: GameAction,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self.bindings.get(&action) {
            Some(InputBinding::Key(key)) => keyboard.just_pressed(*key),
            Some(InputBinding::Mouse(btn)) => mouse.just_pressed(*btn),
            None => false,
        }
    }

    /// Check if action was just released this frame
    pub fn just_released(
        &self,
        action: GameAction,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self.bindings.get(&action) {
            Some(InputBinding::Key(key)) => keyboard.just_released(*key),
            Some(InputBinding::Mouse(btn)) => mouse.just_released(*btn),
            None => false,
        }
    }
}
