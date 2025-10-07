//! Death/Respawn Screen Component
//!
//! Provides a death screen overlay with automatic respawn timer.
//! When the player dies, this screen darkens the view and displays
//! a countdown before automatically respawning the player.

use crate::text::draw_simple_text;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::{Duration, Instant};

/// Configuration for death screen appearance
#[derive(Debug, Clone)]
pub struct DeathScreenStyle {
    /// Overlay darkness (0-255, higher = darker)
    pub overlay_alpha: u8,

    /// "YOU DIED" text color
    pub title_color: Color,

    /// Respawn timer text color
    pub timer_color: Color,

    /// Instruction text color
    pub instruction_color: Color,
}

impl Default for DeathScreenStyle {
    fn default() -> Self {
        DeathScreenStyle {
            overlay_alpha: 220, // Darker than normal menus
            title_color: Color::RGB(255, 50, 50),       // Red
            timer_color: Color::RGB(255, 255, 100),     // Yellow
            instruction_color: Color::RGB(150, 150, 160), // Gray
        }
    }
}

/// State of the death screen
///
/// Manages the death overlay and respawn countdown timer.
/// Triggers automatically when player dies and auto-respawns after the timer expires.
///
/// # Example
///
/// ```rust
/// use crate::gui::DeathScreen;
///
/// // Create death screen
/// let mut death_screen = DeathScreen::new();
///
/// // When player dies
/// if player.state.is_dead() {
///     death_screen.trigger();
/// }
///
/// // In game loop
/// death_screen.render(&mut canvas)?;
///
/// // Check for respawn
/// if death_screen.should_respawn() {
///     player.respawn(spawn_x, spawn_y);
///     death_screen.reset();
/// }
/// ```
pub struct DeathScreen {
    respawn_duration: Duration,
    death_time: Option<Instant>,
    style: DeathScreenStyle,
}

impl DeathScreen {
    /// Creates a new death screen with 3-second respawn timer
    pub fn new() -> Self {
        DeathScreen {
            respawn_duration: Duration::from_secs(3),
            death_time: None,
            style: DeathScreenStyle::default(),
        }
    }

    /// Creates death screen with custom respawn duration
    #[allow(dead_code)] // Reserved for future game mode configurations
    pub fn with_duration(duration: Duration) -> Self {
        DeathScreen {
            respawn_duration: duration,
            death_time: None,
            style: DeathScreenStyle::default(),
        }
    }

    /// Creates death screen with custom style
    #[allow(dead_code)] // Reserved for future customization
    pub fn with_style(style: DeathScreenStyle) -> Self {
        DeathScreen {
            respawn_duration: Duration::from_secs(3),
            death_time: None,
            style,
        }
    }

    /// Trigger death screen (start timer)
    pub fn trigger(&mut self) {
        self.death_time = Some(Instant::now());
    }

    /// Reset death screen (clear timer)
    pub fn reset(&mut self) {
        self.death_time = None;
    }

    /// Check if death screen is active
    #[allow(dead_code)] // Reserved for future state queries
    pub fn is_active(&self) -> bool {
        self.death_time.is_some()
    }

    /// Check if respawn timer has expired
    pub fn should_respawn(&self) -> bool {
        if let Some(death_time) = self.death_time {
            death_time.elapsed() >= self.respawn_duration
        } else {
            false
        }
    }

    /// Get remaining respawn time in seconds
    pub fn remaining_time(&self) -> f32 {
        if let Some(death_time) = self.death_time {
            let elapsed = death_time.elapsed().as_secs_f32();
            let total = self.respawn_duration.as_secs_f32();
            (total - elapsed).max(0.0)
        } else {
            0.0
        }
    }

    /// Render death screen overlay
    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        if self.death_time.is_none() {
            return Ok(());
        }

        // Dark overlay
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(0, 0, 0, self.style.overlay_alpha));
        canvas.fill_rect(None)?;
        canvas.set_blend_mode(sdl2::render::BlendMode::None);

        // Use logical size (game coordinates), not physical window size
        let (screen_width, screen_height) = canvas.logical_size();
        let center_x = screen_width / 2;
        let center_y = screen_height / 2;

        // "YOU DIED" text (large, centered)
        // Each char is 6px wide at scale 4 = 24px per char
        // "YOU DIED" = 8 chars = 192px total width
        draw_simple_text(
            canvas,
            "YOU DIED",
            (center_x - 96) as i32, // Center horizontally
            (center_y - 60) as i32,
            self.style.title_color,
            4, // Large scale
        )?;

        // Respawn timer (medium, centered below title)
        let remaining = self.remaining_time();
        if remaining > 0.0 {
            let timer_text = format!("Respawning in {:.0}...", remaining.ceil());
            // Estimate width: ~17 chars * 6px * scale 2 = ~204px
            draw_simple_text(
                canvas,
                &timer_text,
                (center_x - 100) as i32,
                (center_y + 10) as i32,
                self.style.timer_color,
                2,
            )?;
        }

        // Instructions (small, bottom center)
        // "ESC to exit" = 11 chars * 6px * scale 1 = ~66px
        draw_simple_text(
            canvas,
            "ESC to exit",
            (center_x - 33) as i32,
            (center_y + 80) as i32,
            self.style.instruction_color,
            1,
        )?;

        Ok(())
    }
}

impl Default for DeathScreen {
    fn default() -> Self {
        Self::new()
    }
}
