//! Inventory UI System
//!
//! Renders the player's inventory, including the hotbar and the main inventory window.
//! Follows the Screen-Space GUI pattern.

use crate::item::ItemStack;

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
pub struct InventoryUI<'a> {
    pub is_open: bool,
    style: InventoryUIStyle,
    pub held_item: Option<ItemStack>,
    item_textures: &'a HashMap<String, Texture<'a>>,
    item_registry: &'a ItemRegistry,
}

impl<'a> InventoryUI<'a> {
    /// Creates a new `InventoryUI` with default styling.
    pub fn new(item_textures: &'a HashMap<String, Texture<'a>>, item_registry: &'a ItemRegistry) -> Self {
        InventoryUI {
            is_open: false,
            style: InventoryUIStyle::default(),
            held_item: None,
            item_textures,
            item_registry,
        }
    }

    /// Renders the inventory UI.
    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        player_inventory: &PlayerInventory,
        selected_hotbar_slot: usize,
        mouse_x: i32,
        mouse_y: i32,
    ) -> Result<(), String> {
        self.render_hotbar(canvas, player_inventory, selected_hotbar_slot)?;

        if self.is_open {
            self.render_inventory_window(canvas, player_inventory)?;
        }

        // Render held item
        if let Some(held_stack) = &self.held_item {
            if let Some(texture) = self.item_textures.get(&held_stack.item_id) {
                let item_size = (INVENTORY_SLOT_SIZE as f32 * 1.2) as u32; // Render slightly bigger
                let item_rect = Rect::new(
                    mouse_x - (item_size / 2) as i32, // Center on mouse
                    mouse_y - (item_size / 2) as i32,
                    item_size,
                    item_size,
                );
                canvas.copy(texture, None, item_rect)?;

                // Draw quantity if > 1
                if held_stack.quantity > 1 {
                    let quantity_text = format!("{}", held_stack.quantity);
                    draw_simple_text(
                        canvas,
                        &quantity_text,
                        mouse_x + (item_size / 4) as i32, // Position in bottom-right of held item
                        mouse_y + (item_size / 4) as i32,
                        Color::RGB(255, 255, 255),
                        2,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn render_hotbar(
        &self,
        canvas: &mut Canvas<Window>,
        player_inventory: &PlayerInventory,
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
                if let Some(texture) = self.item_textures.get(&item_stack.item_id) {
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
                if let Some(texture) = self.item_textures.get(&item_stack.item_id) {
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

    pub fn handle_mouse_click(
        &mut self,
        mouse_x: i32,
        mouse_y: i32,
        screen_width: u32,
        screen_height: u32,
        player_inventory: &mut PlayerInventory,
        shift_held: bool,
        mouse_button: sdl2::mouse::MouseButton,
    ) -> Result<(), String> {
        if !self.is_open {
            return Ok(()); // Only handle clicks if inventory is open
        }

        let clicked_slot_index = self.get_slot_at_mouse_pos(mouse_x, mouse_y, screen_width, screen_height);

        match clicked_slot_index {
            Some(index) => {
                match mouse_button {
                    sdl2::mouse::MouseButton::Left => {
                        if shift_held {
                            // Shift-left-click: transfer item
                            if index < HOTBAR_SLOTS { // Clicked on hotbar
                                let item_stack_option = player_inventory.inventory.slots[index].take();
                                if let Some(item_stack) = item_stack_option {
                                    let overflow = player_inventory.inventory.add_item(&item_stack.item_id, item_stack.quantity, self.item_registry)?;
                                    if overflow > 0 {
                                        player_inventory.inventory.slots[index] = Some(ItemStack::new(&item_stack.item_id, overflow));
                                    }
                                }
                            } else { // Clicked on main inventory
                                let item_stack_option = player_inventory.inventory.slots[index].take();
                                if let Some(item_stack) = item_stack_option {
                                    let overflow = player_inventory.inventory.add_item(&item_stack.item_id, item_stack.quantity, self.item_registry)?;
                                    if overflow > 0 {
                                        player_inventory.inventory.slots[index] = Some(ItemStack::new(&item_stack.item_id, overflow));
                                    }
                                }
                            }
                        } else {
                            // Normal left-click: pick up/place/swap
                            if let Some(held_stack) = self.held_item.take() {
                                let current_slot_item = player_inventory.inventory.slots[index].take();
                                player_inventory.inventory.slots[index] = Some(held_stack); // Place held item
                                self.held_item = current_slot_item; // Pick up whatever was in the slot
                            } else {
                                self.held_item = player_inventory.inventory.slots[index].take();
                            }
                        }
                    },
                    sdl2::mouse::MouseButton::Right => {
                        // Right-click: split stack
                        if self.held_item.is_none() {
                            if let Some(slot_stack) = &mut player_inventory.inventory.slots[index] {
                                if slot_stack.quantity > 1 {
                                    self.held_item = slot_stack.split_half();
                                }
                            }
                        }
                    },
                    _ => { /* Ignore other mouse buttons */ }
                }
            }
            None => {
                // Clicked outside any slot
            }
        }

        Ok(())
    }

    // Helper to get the hotbar's bounding rectangle
    fn hotbar_rect(&self, screen_width: u32, screen_height: u32) -> Rect {
        let hotbar_width = (HOTBAR_SLOT_SIZE + HOTBAR_SLOT_MARGIN) * HOTBAR_SLOTS as u32 - HOTBAR_SLOT_MARGIN;
        let start_x = (screen_width - hotbar_width) / 2;
        let start_y = screen_height as i32 - (HOTBAR_SLOT_SIZE as i32 + 15);
        Rect::new(start_x as i32, start_y, hotbar_width, HOTBAR_SLOT_SIZE)
    }

    // Helper to get the main inventory window's bounding rectangle
    fn inventory_window_rect(&self, screen_width: u32, screen_height: u32) -> Rect {
        let inventory_width = (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_MARGIN) * 9 - INVENTORY_SLOT_MARGIN + 2 * INVENTORY_SLOT_MARGIN;
        let inventory_height = (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_MARGIN) * 3 - INVENTORY_SLOT_MARGIN + 2 * INVENTORY_SLOT_MARGIN;
        let start_x = (screen_width - inventory_width) / 2;
        let start_y = (screen_height - inventory_height) / 2;
        Rect::new(start_x as i32, start_y as i32, inventory_width, inventory_height)
    }

    pub fn is_mouse_over_inventory_window(&self, mouse_x: i32, mouse_y: i32, screen_width: u32, screen_height: u32) -> bool {
        if !self.is_open {
            return false;
        }
        let inv_window_rect = self.inventory_window_rect(screen_width, screen_height);
        inv_window_rect.contains_point(sdl2::rect::Point::new(mouse_x, mouse_y))
    }

    // Returns the Rect for a given slot index (0-8 for hotbar, 9-26 for main inventory)
    pub fn get_slot_rect(&self, slot_index: usize, screen_width: u32, screen_height: u32) -> Option<Rect> {
        if slot_index < HOTBAR_SLOTS { // Hotbar slots
            let hotbar_start_x = (screen_width - ((HOTBAR_SLOT_SIZE + HOTBAR_SLOT_MARGIN) * HOTBAR_SLOTS as u32 - HOTBAR_SLOT_MARGIN)) / 2;
            let hotbar_start_y = screen_height as i32 - (HOTBAR_SLOT_SIZE as i32 + 15);
            let slot_x = hotbar_start_x as i32 + (slot_index as i32 * (HOTBAR_SLOT_SIZE + HOTBAR_SLOT_MARGIN) as i32);
            Some(Rect::new(slot_x, hotbar_start_y, HOTBAR_SLOT_SIZE, HOTBAR_SLOT_SIZE))
        } else if self.is_open && slot_index < 27 { // Main inventory slots (only if open)
            let inv_window_rect = self.inventory_window_rect(screen_width, screen_height);
            let local_index = slot_index - HOTBAR_SLOTS; // 0-17 for main inventory
            let row = local_index / 9;
            let col = local_index % 9;
            let slot_x = inv_window_rect.x() + INVENTORY_SLOT_MARGIN as i32 + (col as i32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_MARGIN) as i32);
            let slot_y = inv_window_rect.y() + INVENTORY_SLOT_MARGIN as i32 + (row as i32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_MARGIN) as i32);
            Some(Rect::new(slot_x, slot_y, INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE))
        } else {
            None
        }
    }

    // Returns the slot index at a given mouse position, or None if not over a slot
    pub fn get_slot_at_mouse_pos(&self, mouse_x: i32, mouse_y: i32, screen_width: u32, screen_height: u32) -> Option<usize> {
        // Check hotbar slots first
        for i in 0..HOTBAR_SLOTS {
            if let Some(slot_rect) = self.get_slot_rect(i, screen_width, screen_height) {
                if slot_rect.contains_point(sdl2::rect::Point::new(mouse_x, mouse_y)) {
                    return Some(i);
                }
            }
        }

        // Check main inventory slots (only if open)
        if self.is_open {
            for i in HOTBAR_SLOTS..27 {
                if let Some(slot_rect) = self.get_slot_rect(i, screen_width, screen_height) {
                    if slot_rect.contains_point(sdl2::rect::Point::new(mouse_x, mouse_y)) {
                        return Some(i);
                    }
                }
            }
        }

        None
    }
}