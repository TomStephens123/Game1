use std::fmt;

/// Errors that can occur during inventory operations
#[derive(Debug, Clone)]
pub enum InventoryError {
    /// Slot index out of bounds
    InvalidSlot(usize),

    /// Item ID doesn't exist in registry
    InvalidItem(String),

    /// Slot is occupied (can't place different item)
    SlotOccupied(usize),

    /// Inventory is full (can't add more items)
    InventoryFull,

    /// Tried to remove more items than exist
    InsufficientItems {
        requested: u32,
        available: u32,
    },
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InventoryError::InvalidSlot(index) => {
                write!(f, "Invalid slot index: {}", index)
            }
            InventoryError::InvalidItem(id) => {
                write!(f, "Invalid item ID: {}", id)
            }
            InventoryError::SlotOccupied(index) => {
                write!(f, "Slot {} is occupied", index)
            }
            InventoryError::InventoryFull => {
                write!(f, "Inventory is full")
            }
            InventoryError::InsufficientItems { requested, available } => {
                write!(f, "Insufficient items (requested: {}, available: {})", requested, available)
            }
        }
    }
}

impl std::error::Error for InventoryError {}

impl From<InventoryError> for String {
    fn from(error: InventoryError) -> Self {
        error.to_string()
    }
}
