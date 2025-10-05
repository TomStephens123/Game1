//! Save manager for handling save/load operations
//!
//! This module provides the SaveManager struct which handles:
//! - Saving game state to files
//! - Loading game state from files
//! - Autosave timing
//! - Save file management (listing, cleanup)

use super::types::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct SaveManager {
    save_directory: PathBuf,
    current_save_slot: u8,
    autosave_interval: std::time::Duration,
    last_autosave: Option<SystemTime>,
}

impl SaveManager {
    /// Creates a new SaveManager with the given save directory
    ///
    /// The save directory will be created if it doesn't exist.
    pub fn new(save_directory: impl AsRef<Path>) -> Result<Self, SaveError> {
        let save_dir = save_directory.as_ref().to_path_buf();

        // Create save directory if it doesn't exist
        if !save_dir.exists() {
            fs::create_dir_all(&save_dir)?;
        }

        Ok(SaveManager {
            save_directory: save_dir,
            current_save_slot: 1,  // Default to slot 1
            autosave_interval: std::time::Duration::from_secs(300), // 5 minutes
            last_autosave: None,
        })
    }

    /// Sets the current save slot (1-5)
    pub fn set_save_slot(&mut self, slot: u8) {
        self.current_save_slot = slot.clamp(1, 5);
    }

    /// Gets the current save slot
    pub fn get_save_slot(&self) -> u8 {
        self.current_save_slot
    }

    /// Save the game state to a file
    pub fn save_game(
        &mut self,
        save_file: &SaveFile,
    ) -> Result<PathBuf, SaveError> {
        let filename = self.generate_filename(&save_file.metadata.save_type, save_file.metadata.save_slot);
        let filepath = self.save_directory.join(&filename);

        // Serialize to JSON (pretty format for readability/debugging)
        let json = serde_json::to_string_pretty(save_file)?;

        // Write to file
        fs::write(&filepath, json)?;

        if matches!(save_file.metadata.save_type, SaveType::Auto) {
            self.last_autosave = Some(SystemTime::now());
        }

        println!("Game saved to: {}", filepath.display());

        Ok(filepath)
    }

    /// Load a save file from a specific slot
    pub fn load_game(&self, slot: u8) -> Result<SaveFile, SaveError> {
        let filename = format!("slot_{}.json", slot);
        self.load_game_by_filename(&filename)
    }

    /// Load a save file by filename
    pub fn load_game_by_filename(&self, filename: &str) -> Result<SaveFile, SaveError> {
        let filepath = self.save_directory.join(filename);

        if !filepath.exists() {
            return Err(SaveError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Save file not found: {}", filename),
            )));
        }

        let json = fs::read_to_string(&filepath)?;
        let save_file: SaveFile = serde_json::from_str(&json)?;

        // Version check
        if save_file.version > CURRENT_SAVE_VERSION {
            return Err(SaveError::InvalidVersion(save_file.version));
        }

        Ok(save_file)
    }

    /// Check if autosave is needed
    pub fn should_autosave(&self) -> bool {
        if let Some(last_save) = self.last_autosave {
            if let Ok(elapsed) = SystemTime::now().duration_since(last_save) {
                return elapsed >= self.autosave_interval;
            }
        }
        true // Save if we've never autosaved
    }

    /// List all save files
    pub fn list_saves(&self) -> Result<Vec<SaveFileInfo>, SaveError> {
        let mut saves = Vec::new();

        for entry in fs::read_dir(&self.save_directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    if let Ok(save_file) = self.load_game_by_filename(filename) {
                        saves.push(SaveFileInfo {
                            filename: filename.to_string(),
                            timestamp: save_file.timestamp,
                            metadata: save_file.metadata,
                        });
                    }
                }
            }
        }

        // Sort by timestamp, newest first
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(saves)
    }

    fn generate_filename(&self, save_type: &SaveType, slot: u8) -> String {
        match save_type {
            SaveType::Manual | SaveType::QuickSave => {
                format!("slot_{}.json", slot)
            }
            SaveType::Auto => {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                format!("autosave_slot{}_{}.json", slot, timestamp)
            }
        }
    }

    /// Delete old autosaves, keeping only the N most recent per slot
    pub fn cleanup_autosaves(&self, keep_count: usize) -> Result<(), SaveError> {
        // Group autosaves by slot
        for slot in 1..=5u8 {
            let prefix = format!("autosave_slot{}_", slot);

            let mut autosaves: Vec<_> = fs::read_dir(&self.save_directory)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_str()
                        .map(|s| s.starts_with(&prefix))
                        .unwrap_or(false)
                })
                .collect();

            // Sort by modification time, newest first
            autosaves.sort_by_key(|entry| {
                entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .map(|t| std::cmp::Reverse(t))
            });

            // Delete excess autosaves for this slot
            for entry in autosaves.iter().skip(keep_count) {
                fs::remove_file(entry.path())?;
            }
        }

        Ok(())
    }

    /// Check if a save file exists for a given slot
    pub fn save_exists(&self, slot: u8) -> bool {
        let filename = format!("slot_{}.json", slot);
        let filepath = self.save_directory.join(filename);
        filepath.exists()
    }
}

pub struct SaveFileInfo {
    pub filename: String,
    pub timestamp: SystemTime,
    pub metadata: SaveMetadata,
}
