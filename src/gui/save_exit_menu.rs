//! Save/Exit Menu Component
//!
//! Provides a save and exit confirmation menu with two options:
//! - Save and Exit: Saves the game and quits
//! - Cancel: Returns to game

use super::{Menu, MenuItem};
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Options in the save/exit menu
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveExitOption {
    SaveAndExit,
    Cancel,
}

/// State of the save/exit menu
///
/// This is a wrapper around the base Menu component that provides
/// type-safe option handling for the save/exit menu.
///
/// # Example
///
/// ```rust
/// use crate::gui::{SaveExitMenu, SaveExitOption};
///
/// // Create menu
/// let mut menu = SaveExitMenu::new();
///
/// // Navigate
/// menu.navigate_down();
///
/// // Render
/// menu.render(&mut canvas)?;
///
/// // Handle selection
/// match menu.selected_option() {
///     SaveExitOption::SaveAndExit => {
///         save_game();
///         exit();
///     }
///     SaveExitOption::Cancel => {
///         return_to_game();
///     }
/// }
/// ```
pub struct SaveExitMenu {
    menu: Menu,
}

impl SaveExitMenu {
    /// Creates a new save/exit menu
    pub fn new() -> Self {
        let items = vec![
            MenuItem::new("SAVE AND EXIT".to_string()),
            MenuItem::new("CANCEL".to_string()),
        ];

        SaveExitMenu {
            menu: Menu::new("EXIT".to_string(), items),
        }
    }

    /// Navigate up (wraps to bottom)
    pub fn navigate_up(&mut self) {
        self.menu.select_previous();
    }

    /// Navigate down (wraps to top)
    pub fn navigate_down(&mut self) {
        self.menu.select_next();
    }

    /// Get selected option
    pub fn selected_option(&self) -> SaveExitOption {
        match self.menu.selected_index() {
            0 => SaveExitOption::SaveAndExit,
            1 => SaveExitOption::Cancel,
            _ => SaveExitOption::Cancel, // Default to cancel if out of bounds
        }
    }

    /// Render the menu
    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        self.menu.render(canvas)
    }
}

impl Default for SaveExitMenu {
    fn default() -> Self {
        Self::new()
    }
}
