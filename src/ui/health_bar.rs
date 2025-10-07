//! Health bar component for displaying entity health
//!
//! This module provides a health bar that renders above entities using procedural
//! graphics (SDL2 rectangles). Health bars are stateless components that can be
//! reused across multiple entities.
//!
//! # Example
//!
//! ```rust
//! use crate::ui::{HealthBar, HealthBarStyle};
//!
//! // Create once
//! let health_bar = HealthBar::new();
//!
//! // Render for each entity
//! health_bar.render(
//!     &mut canvas,
//!     entity.x,
//!     entity.y,
//!     entity.width * 2,
//!     entity.height * 2,
//!     entity.health_percentage(),
//! )?;
//! ```

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Configuration for health bar appearance
///
/// This struct defines the visual style of a health bar. Create different
/// styles for different entity types (player vs enemy vs boss).
///
/// # Example
///
/// ```rust
/// use sdl2::pixels::Color;
/// use crate::ui::HealthBarStyle;
///
/// // Enemy health bar (purple)
/// let enemy_style = HealthBarStyle {
///     health_color: Color::RGB(150, 0, 150),
///     low_health_color: Color::RGB(200, 0, 0),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct HealthBarStyle {
    /// Bar width in pixels
    pub width: u32,

    /// Bar height in pixels
    pub height: u32,

    /// Vertical offset from entity top (negative = above entity)
    pub offset_y: i32,

    /// Background bar color (shown when health is depleted)
    pub background_color: Color,

    /// Health bar fill color (used when health > 30%)
    pub health_color: Color,

    /// Health bar fill color when health is low (<30%)
    pub low_health_color: Color,

    /// Border color
    pub border_color: Color,

    /// Border thickness in pixels (0 = no border)
    pub border_thickness: u32,

    /// Show bar even when at full health?
    pub show_when_full: bool,
}

impl Default for HealthBarStyle {
    fn default() -> Self {
        HealthBarStyle {
            width: 32,
            height: 6,     // Increased from 4 to 6 (1.5x taller)
            offset_y: 4, // Closer to entity (was -8, now -10 for taller bar)
            background_color: Color::RGB(50, 50, 50), // Dark gray
            health_color: Color::RGB(0, 200, 0),      // Green
            low_health_color: Color::RGB(200, 0, 0),  // Red
            border_color: Color::RGB(0, 0, 0),        // Black
            border_thickness: 1,
            show_when_full: false, // Hide when at full health
        }
    }
}

/// A health bar component that renders above entities
///
/// Health bars are stateless components that can be reused across multiple entities.
/// Create one health bar and call `render()` for each entity that needs one.
///
/// # Architecture
///
/// This follows the world-space HUD component pattern from `docs/ui-system.md`:
/// - Stateless (no entity references stored)
/// - Procedural rendering (SDL2 rectangles)
/// - Style configuration separate from rendering logic
///
/// # Example
///
/// ```rust
/// // Create once
/// let player_health_bar = HealthBar::new();
/// let enemy_health_bar = HealthBar::with_style(HealthBarStyle {
///     health_color: Color::RGB(150, 0, 150),
///     ..Default::default()
/// });
///
/// // Render for each entity
/// for slime in &slimes {
///     slime.render(&mut canvas)?;
///     enemy_health_bar.render(
///         &mut canvas,
///         slime.x,
///         slime.y,
///         slime.width * 2,
///         slime.height * 2,
///         slime.health as f32 / 8.0,
///     )?;
/// }
/// ```
pub struct HealthBar {
    style: HealthBarStyle,
}

impl HealthBar {
    /// Creates a new health bar with default styling
    ///
    /// Default style:
    /// - 32x6 pixels
    /// - Green health color
    /// - Red color when health < 30%
    /// - 10 pixels above entity
    /// - Hidden when at full health
    pub fn new() -> Self {
        HealthBar {
            style: HealthBarStyle::default(),
        }
    }

    /// Creates a health bar with custom styling
    ///
    /// # Example
    ///
    /// ```rust
    /// let boss_health_bar = HealthBar::with_style(HealthBarStyle {
    ///     width: 200,
    ///     height: 20,
    ///     offset_y: 50,  // Below entity
    ///     health_color: Color::RGB(255, 165, 0),  // Orange
    ///     show_when_full: true,  // Always visible
    ///     ..Default::default()
    /// });
    /// ```
    pub fn with_style(style: HealthBarStyle) -> Self {
        HealthBar { style }
    }

    /// Renders the health bar above an entity
    ///
    /// # Parameters
    ///
    /// - `canvas`: SDL2 canvas to render to
    /// - `entity_x`, `entity_y`: Entity's world position (top-left corner)
    /// - `entity_width`, `entity_height`: Entity's rendered size (after sprite scaling)
    /// - `health_percentage`: Current health as 0.0-1.0 (use `stats.health.percentage()`)
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success
    /// - `Err(String)` if SDL2 rendering fails
    ///
    /// # Example
    ///
    /// ```rust
    /// health_bar.render(
    ///     &mut canvas,
    ///     player.x,
    ///     player.y,
    ///     player.width * 2,  // Player sprite scaled 2x
    ///     player.height * 2,
    ///     player.stats.health.percentage(),  // 0.0 to 1.0
    /// )?;
    /// ```
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        entity_x: i32,
        entity_y: i32,
        entity_width: u32,
        _entity_height: u32, // Reserved for future positioning options
        health_percentage: f32,
    ) -> Result<(), String> {
        // Don't render if health is full and show_when_full is false
        if !self.style.show_when_full && health_percentage >= 1.0 {
            return Ok(());
        }

        // Calculate bar position (centered above entity)
        let bar_x = entity_x + (entity_width as i32 / 2) - (self.style.width as i32 / 2);
        let bar_y = entity_y + self.style.offset_y;

        // Background bar (full width, shows depleted health)
        let background_rect = Rect::new(bar_x, bar_y, self.style.width, self.style.height);
        canvas.set_draw_color(self.style.background_color);
        canvas.fill_rect(background_rect)?;

        // Health bar (filled portion)
        let health_width =
            (self.style.width as f32 * health_percentage.clamp(0.0, 1.0)) as u32;

        if health_width > 0 {
            let health_rect = Rect::new(bar_x, bar_y, health_width, self.style.height);

            // Use red color if health is low (<30%), otherwise green
            let fill_color = if health_percentage < 0.3 {
                self.style.low_health_color
            } else {
                self.style.health_color
            };

            canvas.set_draw_color(fill_color);
            canvas.fill_rect(health_rect)?;
        }

        // Border (optional, drawn last so it's on top)
        if self.style.border_thickness > 0 {
            let border_rect = Rect::new(bar_x, bar_y, self.style.width, self.style.height);
            canvas.set_draw_color(self.style.border_color);
            canvas.draw_rect(border_rect)?;
        }

        Ok(())
    }

    /// Updates the health bar's style
    ///
    /// Allows changing the bar's appearance at runtime.
    #[allow(dead_code)] // Reserved for future dynamic style changes
    pub fn set_style(&mut self, style: HealthBarStyle) {
        self.style = style;
    }

    /// Gets a reference to the current style
    #[allow(dead_code)] // Reserved for future style inspection
    pub fn style(&self) -> &HealthBarStyle {
        &self.style
    }
}

impl Default for HealthBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_health_bar_style() {
        let style = HealthBarStyle::default();
        assert_eq!(style.width, 32);
        assert_eq!(style.height, 4);
        assert_eq!(style.offset_y, -8);
        assert!(!style.show_when_full);
    }

    #[test]
    fn test_health_bar_creation() {
        let bar = HealthBar::new();
        assert_eq!(bar.style().width, 32);
    }

    #[test]
    fn test_health_bar_custom_style() {
        let custom_style = HealthBarStyle {
            width: 64,
            height: 8,
            ..Default::default()
        };
        let bar = HealthBar::with_style(custom_style);
        assert_eq!(bar.style().width, 64);
        assert_eq!(bar.style().height, 8);
    }

    #[test]
    fn test_default_trait() {
        let bar1 = HealthBar::new();
        let bar2 = HealthBar::default();
        assert_eq!(bar1.style().width, bar2.style().width);
    }
}
