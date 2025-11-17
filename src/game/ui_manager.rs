// UIManager struct
//
// This module contains the UIManager struct which holds all UI state and components,
// managing menus, HUD elements, and debug overlays.

use crate::gui::{SaveExitMenu, DeathScreen, InventoryUI};
use crate::ui::{HealthBar, FloatingText, BuffDisplay};

use super::DebugMenuState;

/// UIManager holds all UI state and components
/// This struct manages menus, HUD elements, and debug overlays
pub struct UIManager<'a> {
    pub save_exit_menu: SaveExitMenu,
    pub death_screen: DeathScreen,
    pub inventory_ui: InventoryUI<'a>,
    pub player_health_bar: HealthBar,
    pub enemy_health_bar: HealthBar,
    pub floating_text_renderer: FloatingText,
    pub buff_display: BuffDisplay<'a>,
    pub debug_menu_state: DebugMenuState,
    pub show_collision_boxes: bool,
    pub show_tile_grid: bool,
    pub is_tilling: bool,
    pub last_tilled_tile: Option<(i32, i32)>,
    pub mouse_x: i32,
    pub mouse_y: i32,
}
