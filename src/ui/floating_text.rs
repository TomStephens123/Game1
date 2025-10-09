//! Floating text component for displaying temporary messages above entities
//!
//! This module provides floating text (like damage numbers, heal amounts) that
//! renders above entities and animates upward before fading out.
//!
//! # Example
//!
//! ```rust
//! use crate::ui::FloatingText;
//!
//! // Create once (stateless renderer)
//! let floating_text = FloatingText::new();
//!
//! // Render for each active text instance
//! floating_text.render(
//!     &mut canvas,
//!     x, y,           // Current position (updated by animation)
//!     "+2",           // Text to display
//!     Color::RGB(0, 255, 0),  // Green
//!     alpha,          // 0-255 opacity
//! )?;
//! ```

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Configuration for floating text appearance
#[derive(Debug, Clone)]
pub struct FloatingTextStyle {
    /// Font size (as rectangle height in pixels)
    /// We use procedural rendering, so this defines character size
    pub font_size: u32,

    /// Horizontal spacing between characters
    pub char_spacing: i32,

    /// Outline/shadow offset for better visibility
    pub outline_offset: i32,

    /// Outline color (usually black for contrast)
    pub outline_color: Color,
}

impl Default for FloatingTextStyle {
    fn default() -> Self {
        FloatingTextStyle {
            font_size: 12,
            char_spacing: 2,
            outline_offset: 1,
            outline_color: Color::RGB(0, 0, 0),
        }
    }
}

/// A floating text component that renders animated text above entities
///
/// This is a stateless renderer following the world-space HUD pattern.
/// The animation state (position, lifetime) is managed externally.
///
/// # Architecture
///
/// This follows the world-space HUD component pattern:
/// - Stateless (no animation state stored)
/// - Procedural rendering (SDL2 rectangles for simple characters)
/// - Position and alpha passed as parameters
///
/// # Example
///
/// ```rust
/// // In main.rs, track active floating texts
/// struct FloatingTextInstance {
///     x: f32, y: f32,
///     text: String,
///     color: Color,
///     lifetime: f32,
///     max_lifetime: f32,
/// }
///
/// // Update each frame
/// for text in &mut floating_texts {
///     text.y -= 20.0 * delta_time;  // Rise upward
///     text.lifetime += delta_time;
/// }
///
/// // Render each frame
/// for text in &floating_texts {
///     let alpha = ((1.0 - text.lifetime / text.max_lifetime) * 255.0) as u8;
///     floating_text_renderer.render(
///         &mut canvas,
///         text.x as i32, text.y as i32,
///         &text.text,
///         text.color,
///         alpha,
///     )?;
/// }
/// ```
pub struct FloatingText {
    style: FloatingTextStyle,
}

impl FloatingText {
    /// Creates a new floating text renderer with default styling
    pub fn new() -> Self {
        FloatingText {
            style: FloatingTextStyle::default(),
        }
    }

    /// Creates a floating text renderer with custom styling
    pub fn with_style(style: FloatingTextStyle) -> Self {
        FloatingText { style }
    }

    /// Renders floating text at the specified position
    ///
    /// # Parameters
    ///
    /// - `canvas`: SDL2 canvas to render to
    /// - `x`, `y`: Current position of the text (in world coordinates)
    /// - `text`: Text to display (e.g., "+2", "-5", "CRIT!")
    /// - `color`: Text color
    /// - `alpha`: Opacity (0-255, 255 = fully opaque, 0 = transparent)
    ///
    /// # Note
    ///
    /// This uses simple procedural rendering for numbers and basic characters.
    /// For complex fonts, consider using SDL2_ttf in the future.
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        x: i32,
        y: i32,
        text: &str,
        color: Color,
        alpha: u8,
    ) -> Result<(), String> {
        // Apply alpha to color
        let text_color = Color::RGBA(color.r, color.g, color.b, alpha);
        let outline_color = Color::RGBA(
            self.style.outline_color.r,
            self.style.outline_color.g,
            self.style.outline_color.b,
            alpha,
        );

        // Calculate total text width for centering
        let total_width = (text.len() as i32 * (self.style.font_size as i32 + self.style.char_spacing))
            - self.style.char_spacing;
        let start_x = x - (total_width / 2);

        // Render each character
        for (i, ch) in text.chars().enumerate() {
            let char_x = start_x + (i as i32 * (self.style.font_size as i32 + self.style.char_spacing));

            // Draw outline first (shadow effect)
            self.render_char(
                canvas,
                ch,
                char_x + self.style.outline_offset,
                y + self.style.outline_offset,
                outline_color,
            )?;

            // Draw main character
            self.render_char(canvas, ch, char_x, y, text_color)?;
        }

        Ok(())
    }

    /// Renders a single character using procedural graphics
    ///
    /// This is a simple bitmap-style renderer for common characters.
    /// Supports: 0-9, +, -, !, and some letters
    fn render_char(
        &self,
        canvas: &mut Canvas<Window>,
        ch: char,
        x: i32,
        y: i32,
        color: Color,
    ) -> Result<(), String> {
        canvas.set_draw_color(color);

        let size = self.style.font_size as i32;
        let half = size / 2;
        let third = size / 3;

        match ch {
            // Numbers
            '0' => {
                // Rectangle with hollow center
                canvas.draw_rect(Rect::new(x, y, size as u32, size as u32))?;
                canvas.draw_rect(Rect::new(x + 1, y + 1, (size - 2) as u32, (size - 2) as u32))?;
            }
            '1' => {
                // Vertical line
                canvas.fill_rect(Rect::new(x + half - 1, y, 2, size as u32))?;
            }
            '2' => {
                // Top, middle, bottom horizontal + corners
                canvas.fill_rect(Rect::new(x, y, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y + half, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y + size - 2, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x + size - 2, y, 2, (half + 2) as u32))?;
                canvas.fill_rect(Rect::new(x, y + half, 2, (half + 2) as u32))?;
            }
            '3' => {
                // Top, middle, bottom horizontal + right side
                canvas.fill_rect(Rect::new(x, y, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y + half, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y + size - 2, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x + size - 2, y, 2, size as u32))?;
            }
            '4' => {
                // Left top half, middle horizontal, right full
                canvas.fill_rect(Rect::new(x, y, 2, (half + 2) as u32))?;
                canvas.fill_rect(Rect::new(x, y + half, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x + size - 2, y, 2, size as u32))?;
            }
            '5' => {
                // Mirror of 2
                canvas.fill_rect(Rect::new(x, y, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y + half, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y + size - 2, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y, 2, (half + 2) as u32))?;
                canvas.fill_rect(Rect::new(x + size - 2, y + half, 2, (half + 2) as u32))?;
            }
            '6' => {
                // Like 5 but with left bottom
                canvas.fill_rect(Rect::new(x, y, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y + half, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y + size - 2, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y, 2, size as u32))?;
                canvas.fill_rect(Rect::new(x + size - 2, y + half, 2, (half + 2) as u32))?;
            }
            '7' => {
                // Top horizontal + right side
                canvas.fill_rect(Rect::new(x, y, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x + size - 2, y, 2, size as u32))?;
            }
            '8' => {
                // Full rectangle with middle bar
                canvas.draw_rect(Rect::new(x, y, size as u32, size as u32))?;
                canvas.draw_rect(Rect::new(x + 1, y + 1, (size - 2) as u32, (size - 2) as u32))?;
                canvas.fill_rect(Rect::new(x, y + half, size as u32, 2))?;
            }
            '9' => {
                // Like 6 but top instead of bottom
                canvas.fill_rect(Rect::new(x, y, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y + half, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y + size - 2, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y, 2, (half + 2) as u32))?;
                canvas.fill_rect(Rect::new(x + size - 2, y, 2, size as u32))?;
            }

            // Symbols
            '+' => {
                // Horizontal and vertical bars
                canvas.fill_rect(Rect::new(x, y + half - 1, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x + half - 1, y, 2, size as u32))?;
            }
            '-' => {
                // Horizontal bar
                canvas.fill_rect(Rect::new(x, y + half - 1, size as u32, 2))?;
            }
            '!' => {
                // Vertical line with dot at bottom
                canvas.fill_rect(Rect::new(x + half - 1, y, 2, (size - third) as u32))?;
                canvas.fill_rect(Rect::new(x + half - 1, y + size - third / 2, 2, 2))?;
            }

            // Letters (add more as needed)
            'A' => {
                canvas.fill_rect(Rect::new(x, y + third, 2, (size - third) as u32))?;
                canvas.fill_rect(Rect::new(x + size - 2, y + third, 2, (size - third) as u32))?;
                canvas.fill_rect(Rect::new(x, y, size as u32, 2))?;
                canvas.fill_rect(Rect::new(x, y + half, size as u32, 2))?;
            }

            // Default: small rectangle for unknown chars
            _ => {
                canvas.fill_rect(Rect::new(x + third, y + third, (third) as u32, (third) as u32))?;
            }
        }

        Ok(())
    }
}

impl Default for FloatingText {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_floating_text() {
        let text = FloatingText::new();
        assert_eq!(text.style.font_size, 12);
    }

    #[test]
    fn test_custom_style() {
        let style = FloatingTextStyle {
            font_size: 20,
            ..Default::default()
        };
        let text = FloatingText::with_style(style);
        assert_eq!(text.style.font_size, 20);
    }
}
