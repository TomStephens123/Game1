//! World-Space HUD Components
//!
//! This module provides UI elements that render above entities in world coordinates.
//! These components follow the stateless, procedural rendering pattern defined in
//! `docs/ui-system.md`.
//!
//! # Architecture
//!
//! World-space HUD components are **stateless rendering components** that:
//! - Use world coordinates (entity x, y positions)
//! - Render above entities in the game world
//! - Are created once and reused across all entities
//! - Use procedural rendering (SDL2 primitives)
//!
//! See `docs/ui-system.md` for the full architecture and design philosophy.
//!
//! # Available Components
//!
//! - [`HealthBar`] - Displays health above entities
//! - [`FloatingText`] - Floating text for damage/heal numbers
//! - [`BuffDisplay`] - Screen-space buff indicator display
//!
//! # Future Components (see docs/ui-system.md)
//!
//! - `NameTag` - Entity name labels
//! - `InteractionPrompt` - "Press E" text above interactables
//!
//! # Example Usage
//!
//! ```rust
//! use crate::ui::{HealthBar, HealthBarStyle};
//! use sdl2::pixels::Color;
//!
//! // Create health bars once (in main.rs)
//! let player_health_bar = HealthBar::new();
//! let enemy_health_bar = HealthBar::with_style(HealthBarStyle {
//!     health_color: Color::RGB(150, 0, 150),  // Purple for enemies
//!     ..Default::default()
//! });
//!
//! // In render loop, call render for each entity
//! player_health_bar.render(
//!     &mut canvas,
//!     player.x,
//!     player.y,
//!     player.width * 2,  // Account for sprite scaling
//!     player.height * 2,
//!     player.stats.health.percentage(),
//! )?;
//! ```

pub mod health_bar;
pub mod floating_text;
pub mod buff_display;

pub use health_bar::{HealthBar, HealthBarStyle};
pub use floating_text::{FloatingText}; //, FloatingTextStyle};
pub use buff_display::{BuffDisplay}; //, BuffDisplayStyle};
