//! Screen-Space GUI System
//!
//! This module provides UI elements that render at fixed screen positions,
//! independent of world entities. These components follow the Screen-Space GUI
//! pattern defined in `docs/ui-system.md` System 2.
//!
//! # Architecture
//!
//! Screen-space GUI elements:
//! - Use screen coordinates (pixels from screen edges)
//! - Render on top layer (above world-space HUD)
//! - Are stateful (windows can be open/closed, have content)
//! - Use procedural rendering (SDL2 primitives)
//!
//! # Available Components
//!
//! - [`SaveExitMenu`] - Save and exit confirmation menu
//! - [`DeathScreen`] - Death screen with respawn timer
//!
//! # Example Usage
//!
//! ```rust
//! use crate::gui::{SaveExitMenu, SaveExitOption};
//!
//! // Create menu once
//! let mut save_exit_menu = SaveExitMenu::new();
//!
//! // Handle input
//! save_exit_menu.navigate_down();
//!
//! // Render
//! save_exit_menu.render(&mut canvas)?;
//!
//! // Check selection
//! match save_exit_menu.selected_option() {
//!     SaveExitOption::SaveAndExit => { /* ... */ }
//!     SaveExitOption::Cancel => { /* ... */ }
//! }
//! ```

pub mod menu;
pub mod save_exit_menu;
pub mod death_screen;

pub use menu::{Menu, MenuItem};
pub use save_exit_menu::{SaveExitMenu, SaveExitOption};
pub use death_screen::DeathScreen;
