//! Bitmap Text Rendering
//!
//! This module provides procedural text rendering using a 5x7 bitmap font.
//! Characters are rendered using SDL2 rectangles, supporting scaling and colors.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Renders bitmap text using procedural rectangles (5x7 font)
///
/// # Parameters
///
/// - `canvas`: SDL2 canvas to render to
/// - `text`: Text string to render (case-insensitive)
/// - `x`, `y`: Top-left position in pixels
/// - `color`: Text color
/// - `scale`: Scaling factor (1 = 5x7 pixels, 2 = 10x14 pixels, etc.)
///
/// # Returns
///
/// - `Ok(())` on success
/// - `Err(String)` if SDL2 rendering fails
///
/// # Example
///
/// ```rust
/// use sdl2::pixels::Color;
///
/// draw_simple_text(
///     &mut canvas,
///     "HELLO WORLD",
///     100,
///     50,
///     Color::RGB(255, 255, 255),
///     2,  // 2x scale = 10x14 pixel characters
/// )?;
/// ```
pub fn draw_simple_text(
    canvas: &mut Canvas<Window>,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
    scale: u32,
) -> Result<(), String> {
    canvas.set_draw_color(color);

    let char_width = 6 * scale; // 5 pixels + 1 spacing
    let pixel_size = scale as i32;

    for (i, c) in text.chars().enumerate() {
        let char_x = x + (i as i32 * char_width as i32);

        // 5x7 bitmap font patterns (1 = pixel on, 0 = pixel off)
        let pattern: &[u8] = match c.to_ascii_uppercase() {
            'A' => &[0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'B' => &[0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
            'C' => &[0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
            'D' => &[0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
            'E' => &[0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
            'F' => &[0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
            'G' => &[0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
            'H' => &[0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'I' => &[0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111],
            'J' => &[0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
            'K' => &[0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
            'L' => &[0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
            'M' => &[0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001],
            'N' => &[0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
            'O' => &[0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'P' => &[0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
            'Q' => &[0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
            'R' => &[0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
            'S' => &[0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
            'T' => &[0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
            'U' => &[0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'V' => &[0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
            'W' => &[0b10001, 0b10001, 0b10001, 0b10001, 0b10101, 0b11011, 0b10001],
            'X' => &[0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
            'Y' => &[0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
            'Z' => &[0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
            '0' => &[0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
            '1' => &[0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
            '2' => &[0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
            '3' => &[0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
            '4' => &[0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
            '5' => &[0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
            '6' => &[0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
            '7' => &[0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
            '8' => &[0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
            '9' => &[0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
            ':' => &[0b00000, 0b00000, 0b00100, 0b00000, 0b00100, 0b00000, 0b00000],
            '/' => &[0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
            '<' => &[0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010],
            '>' => &[0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000],
            '-' => &[0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
            '+' => &[0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
            '.' => &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
            '!' => &[0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
            '(' => &[0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
            ')' => &[0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
            ' ' => &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
            _ => &[0b11111, 0b11111, 0b11111, 0b11111, 0b11111, 0b11111, 0b11111], // Full block for unknown
        };

        // Draw the character pixel by pixel
        for (row, &pattern_row) in pattern.iter().enumerate() {
            for col in 0..5 {
                if (pattern_row >> (4 - col)) & 1 == 1 {
                    canvas.fill_rect(Rect::new(
                        char_x + (col * pixel_size),
                        y + (row as i32 * pixel_size),
                        scale,
                        scale,
                    ))?;
                }
            }
        }
    }

    Ok(())
}
