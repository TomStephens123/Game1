//! Inventory UI System
//!
//! Renders the player's inventory, including the hotbar and the main inventory window.
//! Follows the Screen-Space GUI pattern.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::inventory::player::PlayerInventory;
use crate::item::registry::ItemRegistry;
use crate::text::draw_simple_text;

use std::collections::HashMap;

const HOTBAR_SLOT_SIZE: u32 = 36;
const HOTBAR_SLOT_MARGIN: u32 = 4;
const INVENTORY_SLOT_SIZE: u32 = 64;
const INVENTORY_SLOT_MARGIN: u32 = 4;
const HOTBAR_SLOTS: usize = 9;

/// Represents the visual style of the inventory UI.
#[derive(Debug, Clone)]
pub struct InventoryUIStyle {
    pub background_color: Color,
    pub border_color: Color,
    pub slot_color: Color,
    pub selected_slot_color: Color,
}

impl Default for InventoryUIStyle {
    fn default() -> Self {
        InventoryUIStyle {
            background_color: Color::RGBA(25, 25, 35, 51),
            border_color: Color::RGBA(80, 80, 100, 220),
            slot_color: Color::RGBA(50, 50, 60, 200),
            selected_slot_color: Color::RGBA(255, 255, 100, 255),
        }
    }
}

/// Manages the rendering of the inventory UI.
pub struct InventoryUI {
    pub is_open: bool,
    style: InventoryUIStyle,
}

impl InventoryUI {
    /// Creates a new `InventoryUI` with default styling.
    pub fn new() -> Self {
        InventoryUI {
            is_open: false,
            style: InventoryUIStyle::default(),
        }
    }

    /// Renders the inventory UI.
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        player_inventory: &PlayerInventory,
        item_registry: &ItemRegistry,
        item_textures: &HashMap<String, Texture>,
        selected_hotbar_slot: usize,
    ) -> Result<(), String> {
        self.render_hotbar(canvas, player_inventory, item_registry, item_textures, selected_hotbar_slot)?;

        if self.is_open {
            self.render_inventory_window(canvas, player_inventory, item_registry, item_textures)?;
        }

        Ok(())
    }

    /// Renders the hotbar at the bottom of the screen.
    fn render_hotbar(
        &self,
        canvas: &mut Canvas<Window>,
        player_inventory: &PlayerInventory,
        item_registry: &ItemRegistry,
        item_textures: &HashMap<String, Texture>,
        selected_hotbar_slot: usize,
    ) -> Result<(), String> {
        let (screen_width, screen_height) = canvas.logical_size();
        let hotbar_width = (HOTBAR_SLOT_SIZE + HOTBAR_SLOT_MARGIN) * HOTBAR_SLOTS as u32 - HOTBAR_SLOT_MARGIN;
        let start_x = (screen_width - hotbar_width) / 2;
        let start_y = screen_height as i32 - (HOTBAR_SLOT_SIZE as i32 + 15); // 15px from bottom

        for i in 0..HOTBAR_SLOTS {
            let slot_x = start_x as i32 + (i as i32 * (HOTBAR_SLOT_SIZE + HOTBAR_SLOT_MARGIN) as i32);
            let slot_rect = Rect::new(slot_x, start_y, HOTBAR_SLOT_SIZE, HOTBAR_SLOT_SIZE);

            // Draw slot background
            canvas.set_draw_color(self.style.slot_color);
            canvas.fill_rect(slot_rect)?;

            // Draw border (highlight if selected)
            let border_color = if i == selected_hotbar_slot {
                self.style.selected_slot_color
            } else {
                self.style.border_color
            };
            canvas.set_draw_color(border_color);
            canvas.draw_rect(slot_rect)?;

            // Draw item sprite and stack count
            if let Some(item_stack) = &player_inventory.inventory.slots[i] {
                if let Some(texture) = item_textures.get(&item_stack.item_id) {
                    // Center the item sprite within the slot
                    let item_size = HOTBAR_SLOT_SIZE - 8; // 4px padding on each side
                    let item_rect = Rect::new(
                        slot_rect.x() + 4,
                        slot_rect.y() + 4,
                        item_size,
                        item_size,
                    );
                    canvas.copy(texture, None, item_rect)?;

                    // Draw stack count if > 1
                    if item_stack.quantity > 1 {
                        let quantity_text = format!("{}", item_stack.quantity);
                        draw_simple_text(
                            canvas,
                            &quantity_text,
                            slot_rect.x() + 20, // Position in bottom-right
                            slot_rect.y() + 22,
                            Color::RGB(255, 255, 255),
                            1, // Scale
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Renders the main inventory window.
    fn render_inventory_window(
        &self,
        canvas: &mut Canvas<Window>,
        player_inventory: &PlayerInventory,
        item_registry: &ItemRegistry,
        item_textures: &HashMap<String, Texture>,
    ) -> Result<(), String> {
        let (screen_width, screen_height) = canvas.logical_size();
        let inventory_width = (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_MARGIN) * 9 - INVENTORY_SLOT_MARGIN + 2 * INVENTORY_SLOT_MARGIN;
        let inventory_height = (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_MARGIN) * 3 - INVENTORY_SLOT_MARGIN + 2 * INVENTORY_SLOT_MARGIN;
        let start_x = (screen_width - inventory_width) / 2;
        let start_y = (screen_height - inventory_height) / 2;

        // Draw inventory background
        let bg_rect = Rect::new(start_x as i32, start_y as i32, inventory_width, inventory_height);
        canvas.set_draw_color(self.style.background_color);
        canvas.fill_rect(bg_rect)?;
        canvas.set_draw_color(self.style.border_color);
        canvas.draw_rect(bg_rect)?;

        // Draw inventory slots (18 main inventory slots)
        for i in 9..27 {
            let row = (i - 9) / 9;
            let col = (i - 9) % 9;
            let slot_x = start_x as i32 + INVENTORY_SLOT_MARGIN as i32 + (col as i32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_MARGIN) as i32);
            let slot_y = start_y as i32 + INVENTORY_SLOT_MARGIN as i32 + (row as i32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_MARGIN) as i32);
            let slot_rect = Rect::new(slot_x, slot_y, INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE);

            canvas.set_draw_color(self.style.slot_color);
            canvas.fill_rect(slot_rect)?;
            canvas.set_draw_color(self.style.border_color);
            canvas.draw_rect(slot_rect)?;

            if let Some(item_stack) = &player_inventory.inventory.slots[i] {
                if let Some(texture) = item_textures.get(&item_stack.item_id) {
                    let item_size = INVENTORY_SLOT_SIZE - 16;
                    let item_rect = Rect::new(
                        slot_rect.x() + 8,
                        slot_rect.y() + 8,
                        item_size,
                        item_size,
                    );
                    canvas.copy(texture, None, item_rect)?;

                    if item_stack.quantity > 1 {
                        let quantity_text = format!("{}", item_stack.quantity);
                        draw_simple_text(
                            canvas,
                            &quantity_text,
                            slot_rect.x() + 40,
                            slot_rect.y() + 40,
                            Color::RGB(255, 255, 255),
                            2,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }
}