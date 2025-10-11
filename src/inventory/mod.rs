// Inventory system module
//
// This module provides inventory management for Game1, including:
// - Generic inventory container with slots
// - Player inventory with hotbar
// - Container inventories for blocks (chests, etc.)

pub mod error;
pub mod inventory;
pub mod player;

// Re-export main types
pub use player::PlayerInventory;
