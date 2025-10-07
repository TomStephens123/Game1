//! Base Menu Component
//!
//! Provides a reusable overlay menu component for screen-space GUI.
//! Supports keyboard navigation, customizable styling, and selection highlighting.

use crate::text::draw_simple_text;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Configuration for menu appearance
#[derive(Debug, Clone)]
pub struct MenuStyle {
    /// Menu box width in pixels
    pub width: u32,

    /// Menu box height in pixels
    pub height: u32,

    /// Background color
    pub background_color: Color,

    /// Border color
    pub border_color: Color,

    /// Border thickness (draws double border if > 1)
    pub border_thickness: u32,

    /// Overlay darkness (0-255, higher = darker)
    pub overlay_alpha: u8,

    /// Title text color
    pub title_color: Color,

    /// Normal item text color
    pub item_color: Color,

    /// Selected item text color
    pub selected_item_color: Color,

    /// Selection highlight color
    pub highlight_color: Color,
}

impl Default for MenuStyle {
    fn default() -> Self {
        MenuStyle {
            width: 500,
            height: 240,
            background_color: Color::RGB(30, 30, 40),
            border_color: Color::RGB(100, 100, 120),
            border_thickness: 2,
            overlay_alpha: 180,
            title_color: Color::RGB(220, 220, 240),
            item_color: Color::RGB(160, 160, 170),
            selected_item_color: Color::RGB(255, 255, 255),
            highlight_color: Color::RGB(80, 100, 140),
        }
    }
}

/// A menu item with text and enabled state
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub text: String,
    #[allow(dead_code)] // Reserved for disabled menu item styling
    pub enabled: bool,
}

impl MenuItem {
    /// Creates a new enabled menu item
    pub fn new(text: String) -> Self {
        MenuItem {
            text,
            enabled: true,
        }
    }
}

/// A stateful overlay menu component
///
/// This is a reusable menu that can be used for various screen-space overlays
/// (save/exit, pause, settings, etc.). The menu handles:
/// - Navigation (up/down selection)
/// - Rendering (overlay, box, items, highlighting)
/// - Style customization
///
/// # Example
///
/// ```rust
/// use crate::gui::{Menu, MenuItem};
///
/// let menu = Menu::new(
///     "EXIT".to_string(),
///     vec![
///         MenuItem::new("SAVE AND EXIT".to_string()),
///         MenuItem::new("CANCEL".to_string()),
///     ],
/// );
///
/// // Navigate
/// menu.select_next();
///
/// // Render
/// menu.render(&mut canvas)?;
///
/// // Check selection
/// let selected = menu.selected_index();
/// ```
pub struct Menu {
    title: String,
    items: Vec<MenuItem>,
    selected_index: usize,
    style: MenuStyle,
}

impl Menu {
    /// Creates a new menu with default styling
    pub fn new(title: String, items: Vec<MenuItem>) -> Self {
        Menu {
            title,
            items,
            selected_index: 0,
            style: MenuStyle::default(),
        }
    }

    /// Creates a menu with custom styling
    #[allow(dead_code)] // Reserved for future custom-styled menus
    pub fn with_style(title: String, items: Vec<MenuItem>, style: MenuStyle) -> Self {
        Menu {
            title,
            items,
            selected_index: 0,
            style,
        }
    }

    /// Move selection up (wraps to bottom)
    pub fn select_previous(&mut self) {
        if self.selected_index == 0 {
            self.selected_index = self.items.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    /// Move selection down (wraps to top)
    pub fn select_next(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.items.len();
    }

    /// Get currently selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Render the menu at screen center
    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // 1. Semi-transparent overlay (darken screen)
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(0, 0, 0, self.style.overlay_alpha));
        canvas.fill_rect(None)?;
        canvas.set_blend_mode(sdl2::render::BlendMode::None);

        // 2. Calculate centered position using logical size (not physical window size)
        let (screen_width, screen_height) = canvas.logical_size();
        let menu_x = (screen_width - self.style.width) / 2;
        let menu_y = (screen_height - self.style.height) / 2;

        // 3. Menu background
        canvas.set_draw_color(self.style.background_color);
        canvas.fill_rect(Rect::new(
            menu_x as i32,
            menu_y as i32,
            self.style.width,
            self.style.height,
        ))?;

        // 4. Double border
        canvas.set_draw_color(self.style.border_color);
        canvas.draw_rect(Rect::new(
            menu_x as i32,
            menu_y as i32,
            self.style.width,
            self.style.height,
        ))?;
        if self.style.border_thickness > 1 {
            canvas.draw_rect(Rect::new(
                (menu_x + 2) as i32,
                (menu_y + 2) as i32,
                self.style.width - 4,
                self.style.height - 4,
            ))?;
        }

        // 5. Title (centered)
        let title_width = self.title.len() as u32 * 6 * 3; // 6 pixels per char * scale 3
        draw_simple_text(
            canvas,
            &self.title,
            (menu_x + (self.style.width - title_width) / 2) as i32,
            (menu_y + 30) as i32,
            self.style.title_color,
            3,
        )?;

        // 6. Menu items
        let item_height = 60;
        let item_start_y = menu_y + 100;

        for (i, item) in self.items.iter().enumerate() {
            let item_y = item_start_y + (i as u32 * item_height);
            let is_selected = i == self.selected_index;

            // Selection highlight
            if is_selected {
                canvas.set_draw_color(self.style.highlight_color);
                canvas.fill_rect(Rect::new(
                    (menu_x + 15) as i32,
                    item_y as i32 - 3,
                    self.style.width - 30,
                    36,
                ))?;
            }

            // Item text
            let text_color = if is_selected {
                self.style.selected_item_color
            } else {
                self.style.item_color
            };

            draw_simple_text(
                canvas,
                &item.text,
                (menu_x + 80) as i32,
                item_y as i32,
                text_color,
                3,
            )?;
        }

        Ok(())
    }
}
