use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::EventPump;

/// Actions the player can perform in the game
///
/// This enum represents all possible high-level game actions that can be
/// triggered by input. It decouples input handling from action execution.
#[derive(Debug, Clone, PartialEq)]
pub enum GameAction {
    // === Combat ===
    Attack,

    // === UI Navigation ===
    OpenInventory,
    CloseInventory,
    OpenDebugMenu,
    CloseDebugMenu,
    OpenExitMenu,
    CloseExitMenu,

    // === Menu Navigation ===
    MenuUp,
    MenuDown,
    MenuLeft(bool),   // bool = shift held (for larger increments)
    MenuRight(bool),  // bool = shift held
    MenuConfirm,

    // === Inventory Actions ===
    UseItem(usize),  // slot index
    DropItem(usize), // slot index
    SelectHotbarSlot(usize), // 0-8 for slots 1-9

    // === Debug Commands ===
    SaveGame,
    LoadGame,
    ToggleCollisionBoxes,
    ToggleGridOverlay,
    TogglePause,

    // === World Interaction ===
    SpawnSlime(i32, i32),  // x, y in world coordinates
    UseHoe(i32, i32),       // x, y for tile editing
    LeftClick(i32, i32, bool),    // x, y, shift_held for UI/world
    LeftClickRelease,       // Mouse button released
    RightClick(i32, i32),   // Generic right click
    MouseMove(i32, i32),    // x, y - track mouse position

    // === System ===
    Quit,
    SaveAndExit,
}

/// Input context determines which actions are available
///
/// Different game states require different input handling. This enum
/// represents the current input mode to filter irrelevant inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputContext {
    /// Normal gameplay - movement, combat, world interaction
    Playing,
    /// Inventory screen is open
    Inventory,
    /// Exit/save menu is open
    ExitMenu,
    /// Death screen is displayed
    DeathScreen,
    /// Debug menu (F3) is open
    DebugMenu,
}

/// Helper struct to pass UI state without full Game borrow
///
/// This avoids needing to borrow the entire Game struct just to check
/// UI state during input processing.
pub struct UIState {
    pub inventory_open: bool,
    pub debug_menu_open: bool,
    pub exit_menu_open: bool,
    pub death_screen_active: bool,
    pub game_state_dead: bool,
    pub game_state_exit_menu: bool,
}

/// InputSystem processes SDL2 events and produces GameActions
///
/// This system decouples raw input (keyboard, mouse) from game logic.
/// It translates SDL2 events into high-level GameAction commands that
/// the game loop can process.
///
/// # Architecture
///
/// Input processing happens in phases:
/// 1. Determine current InputContext (Playing, Inventory, Menu, etc.)
/// 2. Poll SDL2 events
/// 3. Filter events based on context
/// 4. Translate events to GameActions
/// 5. Return actions to game loop for execution
pub struct InputSystem {
    /// Current input context
    pub context: InputContext,
}

impl InputSystem {
    /// Creates a new InputSystem starting in Playing context
    pub fn new() -> Self {
        InputSystem {
            context: InputContext::Playing,
        }
    }

    /// Update the input context based on current game/UI state
    ///
    /// This should be called before poll_events() to ensure correct
    /// input filtering.
    ///
    /// Priority order (highest to lowest):
    /// 1. DeathScreen - blocks all other input
    /// 2. ExitMenu - save/quit menu
    /// 3. Inventory - player inventory
    /// 4. DebugMenu - F3 debug overlay
    /// 5. Playing - normal gameplay
    pub fn update_context(&mut self, ui_state: &UIState) {
        self.context = if ui_state.death_screen_active || ui_state.game_state_dead {
            InputContext::DeathScreen
        } else if ui_state.exit_menu_open || ui_state.game_state_exit_menu {
            InputContext::ExitMenu
        } else if ui_state.inventory_open {
            InputContext::Inventory
        } else if ui_state.debug_menu_open {
            InputContext::DebugMenu
        } else {
            InputContext::Playing
        };
    }

    /// Process SDL2 events and return list of actions to handle
    ///
    /// This is the main entry point for input processing each frame.
    /// It polls all pending SDL2 events and converts them to GameActions.
    ///
    /// # Arguments
    ///
    /// * `event_pump` - SDL2 event pump for polling events
    /// * `shift_held` - Whether shift key is currently held (checked before poll loop)
    ///
    /// # Returns
    ///
    /// Vec of GameActions to be processed by the game loop
    pub fn poll_events(&self, event_pump: &mut EventPump, shift_held: bool) -> Result<Vec<GameAction>, String> {
        let mut actions = Vec::new();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    actions.push(GameAction::Quit);
                }
                Event::KeyDown {
                    keycode: Some(key),
                    keymod,
                    ..
                } => {
                    self.handle_keydown(key, keymod, &mut actions);
                }
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    self.handle_mouse_down(mouse_btn, x, y, shift_held, &mut actions);
                }
                Event::MouseButtonUp {
                    mouse_btn, ..
                } => {
                    self.handle_mouse_up(mouse_btn, &mut actions);
                }
                Event::MouseMotion { x, y, .. } => {
                    actions.push(GameAction::MouseMove(x, y));
                }
                _ => {
                    // Ignore other event types (for now)
                }
            }
        }

        Ok(actions)
    }

    /// Handle keyboard key press events
    ///
    /// Routes key presses to context-specific handlers.
    fn handle_keydown(&self, key: Keycode, keymod: sdl2::keyboard::Mod, actions: &mut Vec<GameAction>) {
        match self.context {
            InputContext::Playing => self.handle_playing_keys(key, actions),
            InputContext::Inventory => self.handle_inventory_keys(key, actions),
            InputContext::ExitMenu => self.handle_exit_menu_keys(key, actions),
            InputContext::DeathScreen => self.handle_death_screen_keys(key, actions),
            InputContext::DebugMenu => self.handle_debug_menu_keys(key, keymod, actions),
        }
    }

    /// Handle keys during normal gameplay
    fn handle_playing_keys(&self, key: Keycode, actions: &mut Vec<GameAction>) {
        match key {
            // Combat
            Keycode::M => actions.push(GameAction::Attack),

            // UI
            Keycode::I => actions.push(GameAction::OpenInventory),
            Keycode::Escape => actions.push(GameAction::OpenExitMenu),
            Keycode::F3 => actions.push(GameAction::OpenDebugMenu),

            // Debug commands
            Keycode::F5 => actions.push(GameAction::SaveGame),
            Keycode::F9 => actions.push(GameAction::LoadGame),
            Keycode::B => actions.push(GameAction::ToggleCollisionBoxes),
            Keycode::G => actions.push(GameAction::ToggleGridOverlay),
            Keycode::P => actions.push(GameAction::TogglePause),

            // Hotbar slot selection
            Keycode::Num1 => actions.push(GameAction::SelectHotbarSlot(0)),
            Keycode::Num2 => actions.push(GameAction::SelectHotbarSlot(1)),
            Keycode::Num3 => actions.push(GameAction::SelectHotbarSlot(2)),
            Keycode::Num4 => actions.push(GameAction::SelectHotbarSlot(3)),
            Keycode::Num5 => actions.push(GameAction::SelectHotbarSlot(4)),
            Keycode::Num6 => actions.push(GameAction::SelectHotbarSlot(5)),
            Keycode::Num7 => actions.push(GameAction::SelectHotbarSlot(6)),
            Keycode::Num8 => actions.push(GameAction::SelectHotbarSlot(7)),
            Keycode::Num9 => actions.push(GameAction::SelectHotbarSlot(8)),

            _ => {
                // Unhandled keys in Playing context
            }
        }
    }

    /// Handle keys when inventory is open
    fn handle_inventory_keys(&self, key: Keycode, actions: &mut Vec<GameAction>) {
        match key {
            // Close inventory
            Keycode::I | Keycode::Escape => {
                actions.push(GameAction::CloseInventory);
            }

            // Use items from hotbar slots
            Keycode::Num1 => actions.push(GameAction::UseItem(0)),
            Keycode::Num2 => actions.push(GameAction::UseItem(1)),
            Keycode::Num3 => actions.push(GameAction::UseItem(2)),
            Keycode::Num4 => actions.push(GameAction::UseItem(3)),
            Keycode::Num5 => actions.push(GameAction::UseItem(4)),
            Keycode::Num6 => actions.push(GameAction::UseItem(5)),
            Keycode::Num7 => actions.push(GameAction::UseItem(6)),
            Keycode::Num8 => actions.push(GameAction::UseItem(7)),

            _ => {
                // Unhandled keys in Inventory context
            }
        }
    }

    /// Handle keys when exit menu is open
    fn handle_exit_menu_keys(&self, key: Keycode, actions: &mut Vec<GameAction>) {
        match key {
            Keycode::Escape => {
                actions.push(GameAction::CloseExitMenu);
            }
            Keycode::Up => {
                actions.push(GameAction::MenuUp);
            }
            Keycode::Down => {
                actions.push(GameAction::MenuDown);
            }
            Keycode::Return | Keycode::Space => {
                actions.push(GameAction::MenuConfirm);
            }
            _ => {
                // Other keys ignored
            }
        }
    }

    /// Handle keys on death screen
    fn handle_death_screen_keys(&self, _key: Keycode, _actions: &mut Vec<GameAction>) {
        // Death screen handles its own input (respawn button, etc.)
    }

    /// Handle keys when debug menu is open
    fn handle_debug_menu_keys(&self, key: Keycode, keymod: sdl2::keyboard::Mod, actions: &mut Vec<GameAction>) {
        use sdl2::keyboard::Mod;

        let shift_held = keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);

        match key {
            Keycode::F3 | Keycode::Escape => {
                actions.push(GameAction::CloseDebugMenu);
            }
            Keycode::Up => {
                actions.push(GameAction::MenuUp);
            }
            Keycode::Down => {
                actions.push(GameAction::MenuDown);
            }
            Keycode::Left => {
                actions.push(GameAction::MenuLeft(shift_held));
            }
            Keycode::Right => {
                actions.push(GameAction::MenuRight(shift_held));
            }
            Keycode::Return | Keycode::Space => {
                actions.push(GameAction::MenuConfirm);
            }
            _ => {
                // Other keys ignored
            }
        }
    }

    /// Handle mouse button press events
    fn handle_mouse_down(
        &self,
        button: MouseButton,
        x: i32,
        y: i32,
        shift_held: bool,
        actions: &mut Vec<GameAction>,
    ) {
        match button {
            MouseButton::Left => {
                actions.push(GameAction::LeftClick(x, y, shift_held));
            }
            MouseButton::Right => {
                actions.push(GameAction::RightClick(x, y));
            }
            _ => {
                // Ignore other mouse buttons
            }
        }
    }

    /// Handle mouse button release events
    fn handle_mouse_up(
        &self,
        button: MouseButton,
        actions: &mut Vec<GameAction>,
    ) {
        match button {
            MouseButton::Left => {
                actions.push(GameAction::LeftClickRelease);
            }
            _ => {
                // Ignore other mouse buttons for now
            }
        }
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_system_creation() {
        let input = InputSystem::new();
        assert_eq!(input.context, InputContext::Playing);
    }

    #[test]
    fn test_context_switching() {
        let mut input = InputSystem::new();
        assert_eq!(input.context, InputContext::Playing);

        // Simulate opening inventory
        let ui_state = UIState {
            inventory_open: true,
            debug_menu_open: false,
            exit_menu_open: false,
            death_screen_active: false,
            game_state_dead: false,
            game_state_exit_menu: false,
        };
        input.update_context(&ui_state);
        assert_eq!(input.context, InputContext::Inventory);

        // Simulate opening debug menu
        let ui_state = UIState {
            inventory_open: false,
            debug_menu_open: true,
            exit_menu_open: false,
            death_screen_active: false,
            game_state_dead: false,
            game_state_exit_menu: false,
        };
        input.update_context(&ui_state);
        assert_eq!(input.context, InputContext::DebugMenu);
    }

    #[test]
    fn test_context_priority() {
        let mut input = InputSystem::new();

        // Death screen has highest priority
        let ui_state = UIState {
            inventory_open: true,
            debug_menu_open: true,
            exit_menu_open: true,
            death_screen_active: false,
            game_state_dead: true,
            game_state_exit_menu: false,
        };
        input.update_context(&ui_state);
        assert_eq!(input.context, InputContext::DeathScreen);

        // Exit menu has next priority
        let ui_state = UIState {
            inventory_open: true,
            debug_menu_open: true,
            exit_menu_open: true,
            death_screen_active: false,
            game_state_dead: false,
            game_state_exit_menu: true,
        };
        input.update_context(&ui_state);
        assert_eq!(input.context, InputContext::ExitMenu);
    }

    #[test]
    fn test_game_action_equality() {
        assert_eq!(GameAction::Attack, GameAction::Attack);
        assert_eq!(GameAction::OpenInventory, GameAction::OpenInventory);
        assert_ne!(GameAction::Attack, GameAction::OpenInventory);

        assert_eq!(GameAction::UseItem(0), GameAction::UseItem(0));
        assert_ne!(GameAction::UseItem(0), GameAction::UseItem(1));
    }
}
