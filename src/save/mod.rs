//! Save/Load system for Game1
//!
//! This module provides a comprehensive save/load system with:
//! - JSON-based save files (human-readable, debuggable)
//! - Multiple save slots (1-5)
//! - Automatic saves every 5 minutes
//! - Extensible trait-based design for new entity types
//!
//! # Architecture
//!
//! - `types`: Save data structures and error types
//! - `manager`: SaveManager for file operations
//! - `saveable`: Saveable trait for entities
//!
//! # Example Usage
//!
//! ```ignore
//! // Create save manager
//! let mut save_manager = SaveManager::new("~/.game1/saves")?;
//!
//! // Save game
//! let save_file = SaveFile {
//!     version: CURRENT_SAVE_VERSION,
//!     timestamp: SystemTime::now(),
//!     metadata: SaveMetadata { /* ... */ },
//!     world_state: WorldSaveData { /* ... */ },
//!     entities: vec![/* player, slimes, etc. */],
//! };
//! save_manager.save_game(&save_file)?;
//!
//! // Load game
//! let loaded = save_manager.load_game(1)?;  // Load slot 1
//! ```

pub mod manager;
pub mod saveable;
pub mod types;

// Re-export commonly used types
pub use manager::SaveManager;
pub use saveable::Saveable;
pub use types::*;
