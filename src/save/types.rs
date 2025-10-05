//! Save data types for Game1
//!
//! This module defines all the data structures used for saving and loading game state.
//! It uses Serde for serialization/deserialization to JSON format.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// The root save file structure
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFile {
    pub version: u32,
    pub timestamp: SystemTime,
    pub metadata: SaveMetadata,
    pub world_state: WorldSaveData,
    pub entities: Vec<EntitySaveData>,
}

/// Metadata about the save
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub game_version: String,
    pub player_name: Option<String>,
    pub playtime_seconds: u64,
    pub save_type: SaveType,
    pub save_slot: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SaveType {
    Manual,
    Auto,
    QuickSave,
}

/// World state data
#[derive(Debug, Serialize, Deserialize)]
pub struct WorldSaveData {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<String>>,  // Serialized TileId as strings
}

/// Entity save data (polymorphic through entity_type)
#[derive(Debug, Serialize, Deserialize)]
pub struct EntitySaveData {
    pub entity_id: u64,
    pub entity_type: String,  // "player", "slime", "goblin", etc.
    pub position: (i32, i32),
    pub data: String,  // JSON for entity-specific data
}

/// Error types for save/load operations
#[derive(Debug)]
pub enum SaveError {
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    InvalidVersion(u32),
    CorruptedData(String),
    #[allow(dead_code)] // Reserved for future entity system
    EntityNotFound(u64),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::IoError(e) => write!(f, "IO error: {}", e),
            SaveError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            SaveError::InvalidVersion(v) => write!(f, "Invalid save version: {}", v),
            SaveError::CorruptedData(msg) => write!(f, "Corrupted save data: {}", msg),
            SaveError::EntityNotFound(id) => write!(f, "Entity not found: {}", id),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<std::io::Error> for SaveError {
    fn from(err: std::io::Error) -> Self {
        SaveError::IoError(err)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(err: serde_json::Error) -> Self {
        SaveError::SerializationError(err)
    }
}

/// Generic wrapper for saveable data
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub data_type: String,
    pub json_data: String,
}

/// Current save file version
pub const CURRENT_SAVE_VERSION: u32 = 1;
