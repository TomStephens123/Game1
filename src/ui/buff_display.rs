//! Buff display component for showing active buffs on screen
//!
//! This module provides a screen-space UI element that displays which buffs
//! are currently active on the player. It renders in a fixed position on screen
//! (typically top-left corner).
//!
//! # Example
//!
//! ```rust
//! use crate::ui::BuffDisplay;
//!
//! // Create once
//! let texture_creator = canvas.texture_creator();
//! let buff_display = BuffDisplay::new(&texture_creator).unwrap();
//!
//! // Render each frame with current active modifiers
//! buff_display.render(&mut canvas, &player.active_modifiers, player.has_regen)?;
//! ```

use crate::stats::{ModifierEffect, StatType};
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

/// Configuration for buff display appearance
#[derive(Debug, Clone)]
pub struct BuffDisplayStyle {
    /// Screen position (top-left corner)
    pub x: i32,
    pub y: i32,

    /// Icon size in pixels
    pub icon_size: u32,

    /// Spacing between icons
    pub icon_spacing: i32,
}

impl Default for BuffDisplayStyle {
    fn default() -> Self {
        BuffDisplayStyle {
            x: 10,
            y: 10,
            icon_size: 32, // Match the asset size
            icon_spacing: 4,
        }
    }
}

/// Represents the type of buff to be displayed.
/// The order of variants is important as it maps to the sprite sheet.
/// Order: Defense, Speed, Attack, Regen
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum BuffType {
    Defense,
    Speed,
    Attack,
    Regen,
}

/// Buff display component for showing active buffs
///
/// This is a screen-space UI component that renders in a fixed position on screen.
/// It displays icon-style representations of active buffs.
pub struct BuffDisplay<'a> {
    style: BuffDisplayStyle,
    texture: Texture<'a>,
}

impl<'a> BuffDisplay<'a> {
    /// Creates a new buff display with default styling.
    /// Requires a `TextureCreator` to load the icon spritesheet.
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Result<Self, String> {
        let texture = texture_creator.load_texture("assets/sprites/icons.png")?;
        Ok(BuffDisplay {
            style: BuffDisplayStyle::default(),
            texture,
        })
    }

    /// Creates a buff display with custom styling.
    pub fn with_style(
        style: BuffDisplayStyle,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<Self, String> {
        let texture = texture_creator.load_texture("assets/sprites/icons.png")?;
        Ok(BuffDisplay { style, texture })
    }

    /// Renders the buff display with current active modifiers and regeneration status.
    ///
    /// # Parameters
    ///
    /// - `canvas`: SDL2 canvas to render to
    /// - `active_modifiers`: Current active stat modifiers from player
    /// - `has_regen`: Whether regeneration is active
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        active_modifiers: &[ModifierEffect],
        has_regen: bool,
    ) -> Result<(), String> {
        let mut active_buffs = Vec::new();

        for modifier in active_modifiers {
            match modifier.stat_type {
                StatType::AttackDamage => active_buffs.push(BuffType::Attack),
                StatType::Defense => active_buffs.push(BuffType::Defense),
                StatType::MovementSpeed => active_buffs.push(BuffType::Speed),
                _ => {}
            }
        }

        if has_regen {
            active_buffs.push(BuffType::Regen);
        }

        if active_buffs.is_empty() {
            return Ok(());
        }

        // Sort and remove duplicates to ensure a consistent render order
        active_buffs.sort_unstable();
        active_buffs.dedup();

        // Render each active buff icon
        let mut current_x = self.style.x;
        for buff_type in active_buffs {
            self.render_icon(canvas, current_x, self.style.y, buff_type)?;
            current_x += self.style.icon_size as i32 + self.style.icon_spacing;
        }

        Ok(())
    }

    /// Renders a single buff icon from the sprite sheet.
    fn render_icon(
        &self,
        canvas: &mut Canvas<Window>,
        x: i32,
        y: i32,
        buff_type: BuffType,
    ) -> Result<(), String> {
        // Determine which part of the spritesheet to use based on the enum order
        let icon_index = buff_type as i32;
        let source_rect = Rect::new(icon_index * 32, 0, 32, 32);

        // Determine where to draw the icon on the screen
        let dest_rect = Rect::new(x, y, self.style.icon_size, self.style.icon_size);

        // Copy the icon from the texture to the canvas
        canvas.copy(&self.texture, source_rect, dest_rect)?;

        Ok(())
    }
}