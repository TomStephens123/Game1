// Item system module
//
// This module provides the core item system for Game1, including:
// - Item definitions and properties
// - Item registry for centralized storage
// - Item stacks for quantity management

pub mod definition;
pub mod properties;
pub mod registry;
pub mod stack;

// Re-export main types for convenient access
pub use definition::ItemDefinition;
pub use properties::{ItemProperties, ConsumableEffect, EquipmentSlot, ToolType};
pub use registry::ItemRegistry;
pub use stack::ItemStack;
