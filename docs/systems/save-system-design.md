# Save/Load System Design

## Overview
This document outlines the architecture for Game1's save/load system, designed to be extensible, maintainable, and Rust-idiomatic. The system handles manual saves (F5, exit menu) and supports serialization of entities (player, enemies) and world state (tile grid).

**Status**: ✅ **IMPLEMENTED** (v0.1.0 - January 2025)

## Current Implementation

### Working Features
- ✅ Auto-load on game startup
- ✅ F5 quick save
- ✅ Exit menu with "Save and Exit" option
- ✅ JSON-based save format
- ✅ Player state persistence (position, stats, health)
- ✅ Slime state persistence (position, health, alive status)
- ✅ World tile grid persistence (grass/dirt tiles)
- ✅ Single save slot (slot_1.json)

### Save Location
- **macOS/Linux**: `~/.game1/saves/slot_1.json`
- **Windows**: `%USERPROFILE%\.game1\saves\slot_1.json`

### User Controls
- **F5**: Quick save (no confirmation)
- **F9**: Reload from save
- **ESC**: Exit menu with "Save and Exit" or "Cancel"
- **Auto-load**: Automatically loads existing save on game start

## Core Design Principles

### 1. **Trait-Based Serialization**
All saveable game objects will implement a `Saveable` trait, making the system extensible without modifying core save/load logic.

```rust
pub trait Saveable {
    /// Convert the object to a saveable representation
    fn to_save_data(&self) -> Result<SaveData, SaveError>;

    /// Restore the object from saved data
    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> where Self: Sized;
}
```

**Benefits:**
- **Extensibility**: Add new entity types without changing save system code
- **Type Safety**: Rust's type system ensures all saveable objects implement required methods
- **Separation of Concerns**: Each entity knows how to save/load itself
- **Rust Pattern**: Traits are the idiomatic way to define shared behavior

### 2. **Versioned Save Format**
Save files include version metadata to handle format evolution gracefully.

```rust
pub struct SaveFile {
    pub version: u32,
    pub timestamp: SystemTime,
    pub metadata: SaveMetadata,
    pub world_state: WorldSaveData,
    pub entities: Vec<EntitySaveData>,
}
```

**Benefits:**
- Future-proof against game updates
- Backward compatibility with older saves
- Clear migration paths when changing data structures

### 3. **Separate Entity and World State**
World data (blocks, chunks) and entity data (player, enemies) are saved separately but in the same file.

**Benefits:**
- Independent evolution of world vs entity systems
- Easier debugging and testing
- Potential for partial saves/loads in future

## Architecture Components

### Component 1: Save Data Types

**File:** `src/save/types.rs`

```rust
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
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SaveType {
    Manual,
    Auto,
    QuickSave,
}

/// World state data
#[derive(Debug, Serialize, Deserialize)]
pub struct WorldSaveData {
    pub seed: Option<u64>,  // For procedural generation
    pub chunks: Vec<ChunkData>,
    pub modified_blocks: Vec<BlockModification>,
    pub world_time: f64,  // In-game time
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkData {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub blocks: Vec<BlockData>,
    pub is_generated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockData {
    pub local_x: u32,
    pub local_y: u32,
    pub block_type: String,  // Extensible: "grass", "stone", "custom_block"
    pub metadata: Option<String>,  // JSON for block-specific data
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockModification {
    pub world_x: i32,
    pub world_y: i32,
    pub block_type: String,
    pub timestamp: SystemTime,
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
    EntityNotFound(u64),
}

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
```

**Rust Learning Point: Serde Serialization**
- `#[derive(Serialize, Deserialize)]`: Automatic JSON/binary serialization
- Serde handles nested structures, enums, Options automatically
- Type-safe: Can't accidentally serialize/deserialize to wrong type

### Component 2: Save Manager

**File:** `src/save/manager.rs`

```rust
use super::types::*;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;

pub struct SaveManager {
    save_directory: PathBuf,
    current_save_file: Option<String>,
    autosave_interval: std::time::Duration,
    last_autosave: Option<SystemTime>,
}

impl SaveManager {
    pub fn new(save_directory: impl AsRef<Path>) -> Result<Self, SaveError> {
        let save_dir = save_directory.as_ref().to_path_buf();

        // Create save directory if it doesn't exist
        if !save_dir.exists() {
            fs::create_dir_all(&save_dir)?;
        }

        Ok(SaveManager {
            save_directory: save_dir,
            current_save_file: None,
            autosave_interval: std::time::Duration::from_secs(300), // 5 minutes
            last_autosave: None,
        })
    }

    /// Save the game state to a file
    pub fn save_game(
        &mut self,
        save_file: &SaveFile,
        filename: Option<&str>,
    ) -> Result<PathBuf, SaveError> {
        let filename = filename.unwrap_or(&self.generate_filename(&save_file.metadata.save_type));
        let filepath = self.save_directory.join(filename);

        // Serialize to JSON (pretty format for readability/debugging)
        let json = serde_json::to_string_pretty(save_file)?;

        // Write to file
        fs::write(&filepath, json)?;

        self.current_save_file = Some(filename.to_string());

        if matches!(save_file.metadata.save_type, SaveType::Auto) {
            self.last_autosave = Some(SystemTime::now());
        }

        Ok(filepath)
    }

    /// Load a save file
    pub fn load_game(&self, filename: &str) -> Result<SaveFile, SaveError> {
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
                if let Ok(save_file) = self.load_game(path.file_name().unwrap().to_str().unwrap()) {
                    saves.push(SaveFileInfo {
                        filename: path.file_name().unwrap().to_string_lossy().to_string(),
                        timestamp: save_file.timestamp,
                        metadata: save_file.metadata,
                    });
                }
            }
        }

        // Sort by timestamp, newest first
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(saves)
    }

    fn generate_filename(&self, save_type: &SaveType) -> String {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        match save_type {
            SaveType::Manual => format!("save_{}.json", timestamp),
            SaveType::Auto => format!("autosave_{}.json", timestamp),
            SaveType::QuickSave => format!("quicksave_{}.json", timestamp),
        }
    }

    /// Delete old autosaves, keeping only the N most recent
    pub fn cleanup_autosaves(&self, keep_count: usize) -> Result<(), SaveError> {
        let mut autosaves: Vec<_> = fs::read_dir(&self.save_directory)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name()
                    .to_str()
                    .map(|s| s.starts_with("autosave_"))
                    .unwrap_or(false)
            })
            .collect();

        // Sort by modification time, newest first
        autosaves.sort_by_key(|entry| {
            entry.metadata()
                .and_then(|m| m.modified())
                .ok()
        });
        autosaves.reverse();

        // Delete excess autosaves
        for entry in autosaves.iter().skip(keep_count) {
            fs::remove_file(entry.path())?;
        }

        Ok(())
    }
}

pub struct SaveFileInfo {
    pub filename: String,
    pub timestamp: SystemTime,
    pub metadata: SaveMetadata,
}

const CURRENT_SAVE_VERSION: u32 = 1;
```

**Rust Learning Points:**
- **Path Handling**: `PathBuf` and `Path` for cross-platform file paths
- **Error Propagation**: `?` operator for clean error handling
- **Option Pattern**: `Option<T>` for nullable values without null pointers
- **Duration**: Type-safe time handling with `std::time::Duration`

### Component 3: Saveable Trait Implementations

**File:** `src/save/saveable.rs`

```rust
use super::types::*;

pub trait Saveable {
    fn to_save_data(&self) -> Result<SaveData, SaveError>;
    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> where Self: Sized;
}

// Example: Player implementation (in src/player.rs)
/*
impl Saveable for Player<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct PlayerData {
            x: i32,
            y: i32,
            health: f32,
            max_health: f32,
            speed: i32,
            direction: String,
            is_attacking: bool,
            inventory: Vec<String>,  // Future: item IDs
        }

        let player_data = PlayerData {
            x: self.x,
            y: self.y,
            health: self.health,
            max_health: self.max_health,
            speed: self.speed,
            direction: format!("{:?}", self.direction),
            is_attacking: self.is_attacking,
            inventory: vec![],  // Placeholder for future inventory system
        };

        Ok(SaveData {
            data_type: "player".to_string(),
            json_data: serde_json::to_string(&player_data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        #[derive(Deserialize)]
        struct PlayerData {
            x: i32,
            y: i32,
            health: f32,
            max_health: f32,
            speed: i32,
            direction: String,
            is_attacking: bool,
            inventory: Vec<String>,
        }

        if data.data_type != "player" {
            return Err(SaveError::CorruptedData(
                format!("Expected player data, got {}", data.data_type)
            ));
        }

        let player_data: PlayerData = serde_json::from_str(&data.json_data)?;

        // Note: Animation controller needs to be reconstructed separately
        // since it contains texture references that can't be serialized
        let mut player = Player::new(
            player_data.x,
            player_data.y,
            32,  // width
            32,  // height
            player_data.speed,
        );

        player.health = player_data.health;
        player.max_health = player_data.max_health;
        // direction and is_attacking are parsed from string

        Ok(player)
    }
}
*/

// Example: Slime implementation (in src/slime.rs)
/*
impl Saveable for Slime<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct SlimeData {
            x: i32,
            y: i32,
            base_y: i32,
            health: f32,
            behavior: String,  // "Idle" or "Jumping"
        }

        let slime_data = SlimeData {
            x: self.x,
            y: self.y,
            base_y: self.base_y,
            health: self.health,
            behavior: format!("{:?}", self.behavior),
        };

        Ok(SaveData {
            data_type: "slime".to_string(),
            json_data: serde_json::to_string(&slime_data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        // Similar to Player implementation
        // Parse JSON, create Slime, restore state
        todo!()
    }
}
*/
```

**Design Decision: Textures Not Saved**
Texture references (`&'a Texture`) cannot be serialized. Instead:
1. Save entity state data only
2. On load, recreate entities with fresh texture references
3. Restore state from save data

This is a common pattern in game development - resources (textures, audio) are loaded separately from game state.

## Integration Plan

### Phase 1: Basic Infrastructure
1. **Create save module structure**
   - `src/save/mod.rs` - Module declaration
   - `src/save/types.rs` - Save data types
   - `src/save/manager.rs` - SaveManager
   - `src/save/saveable.rs` - Saveable trait

2. **Add dependencies to Cargo.toml**
   ```toml
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   chrono = "0.4"  # For timestamp generation
   ```

3. **Create save directory handling**
   - Default: `~/.game1/saves/` on Unix, `AppData/Game1/saves/` on Windows
   - Configurable via game settings

### Phase 2: Entity Save/Load
1. **Implement Saveable for Player**
   - Add health, max_health fields to Player struct
   - Implement to_save_data() and from_save_data()
   - Handle texture recreation on load

2. **Implement Saveable for Slime**
   - Save behavior state, position, health
   - Restore AI state on load

3. **Create entity registry**
   - Map entity type strings to constructor functions
   - Enable polymorphic loading: `entity_registry.create("slime", save_data)`

### Phase 3: World Save/Load
1. **Design world/chunk system** (if not already exists)
   - Chunk-based terrain storage
   - Modified blocks tracking

2. **Implement world serialization**
   - Save only modified chunks (optimization)
   - Track block modifications with timestamps

3. **Implement world deserialization**
   - Load chunks on demand
   - Apply block modifications

### Phase 4: Main Game Integration
1. **Add SaveManager to main game state**
   ```rust
   struct GameState<'a> {
       player: Player<'a>,
       slimes: Vec<Slime<'a>>,
       save_manager: SaveManager,
       // ... other state
   }
   ```

2. **Implement save game logic**
   ```rust
   fn save_game(game_state: &GameState) -> Result<(), SaveError> {
       let mut entities = Vec::new();

       // Save player
       entities.push(EntitySaveData {
           entity_id: 0,
           entity_type: "player".to_string(),
           position: game_state.player.position(),
           data: game_state.player.to_save_data()?.json_data,
       });

       // Save slimes
       for (i, slime) in game_state.slimes.iter().enumerate() {
           entities.push(EntitySaveData {
               entity_id: i as u64 + 1,
               entity_type: "slime".to_string(),
               position: (slime.x, slime.y),
               data: slime.to_save_data()?.json_data,
           });
       }

       let save_file = SaveFile {
           version: CURRENT_SAVE_VERSION,
           timestamp: SystemTime::now(),
           metadata: SaveMetadata {
               game_version: env!("CARGO_PKG_VERSION").to_string(),
               player_name: None,
               playtime_seconds: 0,  // TODO: track this
               save_type: SaveType::Manual,
           },
           world_state: WorldSaveData {
               seed: None,
               chunks: vec![],
               modified_blocks: vec![],
               world_time: 0.0,
           },
           entities,
       };

       game_state.save_manager.save_game(&save_file, None)?;
       Ok(())
   }
   ```

3. **Implement load game logic**
   - Load save file
   - Recreate entities from save data
   - Restore world state

4. **Add autosave to game loop**
   ```rust
   // In main game loop
   if save_manager.should_autosave() {
       if let Err(e) = save_game(&game_state) {
           eprintln!("Autosave failed: {:?}", e);
       }
       save_manager.cleanup_autosaves(5)?;  // Keep 5 most recent
   }
   ```

### Phase 5: UI and Polish
1. **Add save/load menu**
   - List available saves
   - Display save metadata (timestamp, playtime)
   - Delete saves

2. **Add keybindings**
   - F5: Quick save
   - F9: Quick load
   - ESC menu: Save & Quit

3. **Add visual feedback**
   - "Saving..." indicator during save
   - "Game Saved" notification

## Extensibility Examples

### Adding a New Entity Type

**Step 1:** Define the entity struct
```rust
pub struct Goblin<'a> {
    pub x: i32,
    pub y: i32,
    pub health: f32,
    pub aggro_range: i32,
    animation_controller: AnimationController<'a>,
}
```

**Step 2:** Implement Saveable
```rust
impl Saveable for Goblin<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct GoblinData {
            x: i32,
            y: i32,
            health: f32,
            aggro_range: i32,
        }

        let data = GoblinData {
            x: self.x,
            y: self.y,
            health: self.health,
            aggro_range: self.aggro_range,
        };

        Ok(SaveData {
            data_type: "goblin".to_string(),
            json_data: serde_json::to_string(&data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        // Deserialize and recreate
        todo!()
    }
}
```

**Step 3:** Add to entity list in save/load logic
```rust
// In save_game()
for goblin in &game_state.goblins {
    entities.push(EntitySaveData {
        entity_id: next_id(),
        entity_type: "goblin".to_string(),
        position: (goblin.x, goblin.y),
        data: goblin.to_save_data()?.json_data,
    });
}

// In load_game()
match entity_data.entity_type.as_str() {
    "player" => { /* ... */ }
    "slime" => { /* ... */ }
    "goblin" => {
        let goblin = Goblin::from_save_data(&save_data)?;
        game_state.goblins.push(goblin);
    }
    _ => eprintln!("Unknown entity type: {}", entity_data.entity_type),
}
```

**That's it!** No changes to save system core code required.

### Adding New Player Stats

When you implement the player stats system (health, stamina, etc.), just update the PlayerData struct in Player's Saveable implementation:

```rust
#[derive(Serialize, Deserialize)]
struct PlayerData {
    // Existing fields
    x: i32,
    y: i32,

    // New stats
    health: f32,
    max_health: f32,
    stamina: f32,
    max_stamina: f32,
    level: u32,
    experience: u32,

    // Future additions
    equipment: Vec<String>,  // Item IDs
    skills: Vec<String>,     // Learned skills
}
```

The save system automatically handles new fields because Serde serializes all struct fields.

### Adding World Blocks

When you implement terrain editing:

```rust
// Player places a block
fn place_block(&mut self, world_x: i32, world_y: i32, block_type: &str) {
    // ... update world state ...

    // Track modification for saving
    self.world_state.modified_blocks.push(BlockModification {
        world_x,
        world_y,
        block_type: block_type.to_string(),
        timestamp: SystemTime::now(),
    });
}
```

On save, only modified blocks are saved (not the entire world). On load, modifications are reapplied to the base world.

## Performance Considerations

### Save Performance
- **JSON Format**: Human-readable, debuggable, ~1-2MB for typical save
- **Future Optimization**: Switch to bincode for binary serialization (10x smaller, faster)
- **Compression**: Add gzip compression for save files

### Load Performance
- **Lazy Loading**: Load chunks/entities only when needed
- **Background Loading**: Load in separate thread, show loading screen
- **Incremental Saves**: Save only changed data (delta saves)

### Autosave Impact
- Autosave runs every 5 minutes
- Save operation takes ~10-50ms (JSON) or ~1-5ms (binary)
- Minimal gameplay impact
- Consider: Autosave only when player is in safe location (not in combat)

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_player() {
        let player = Player::new(100, 200, 32, 32, 3);
        let save_data = player.to_save_data().unwrap();
        let loaded_player = Player::from_save_data(&save_data).unwrap();

        assert_eq!(player.x, loaded_player.x);
        assert_eq!(player.y, loaded_player.y);
    }

    #[test]
    fn test_save_manager_create_directory() {
        let temp_dir = std::env::temp_dir().join("game1_test_saves");
        let manager = SaveManager::new(&temp_dir).unwrap();
        assert!(temp_dir.exists());
    }

    #[test]
    fn test_autosave_cleanup() {
        // Create 10 autosaves, cleanup to keep 5
        // Verify only 5 remain
    }
}
```

### Integration Tests
- Save a game, load it, verify all state matches
- Test version compatibility (save with v1, load with v2)
- Test corrupted save files (graceful error handling)

## Migration Strategy

When changing save format (e.g., adding new fields):

```rust
pub fn migrate_save_file(save_file: &mut SaveFile) -> Result<(), SaveError> {
    match save_file.version {
        1 => {
            // Version 1 -> 2 migration
            // Add default values for new fields
            save_file.version = 2;
        }
        2 => {
            // Already latest version
        }
        _ => {
            return Err(SaveError::InvalidVersion(save_file.version));
        }
    }
    Ok(())
}
```

**Backward Compatibility Rule**: New versions can load old saves, old versions reject new saves.

## Security Considerations

### Save File Integrity
- **Checksums**: Add SHA-256 hash to detect tampering
- **Validation**: Validate all loaded data (bounds checking, type checking)
- **Sandbox**: Don't execute code from save files (JSON is data only)

### Path Traversal Prevention
```rust
// Sanitize filenames to prevent path traversal
fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .collect()
}
```

## Future Enhancements

### Cloud Saves
- Upload saves to cloud storage (Steam Cloud, custom server)
- Sync across devices
- Requires: encryption, authentication, conflict resolution

### Replay System
- Record player inputs with timestamps
- Replay saved games
- Useful for: debugging, speedruns, sharing gameplay

### Save Compression
```rust
use flate2::write::GzEncoder;

pub fn save_compressed(save_file: &SaveFile, path: &Path) -> Result<(), SaveError> {
    let json = serde_json::to_string(save_file)?;
    let file = File::create(path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(json.as_bytes())?;
    Ok(())
}
```

### Incremental Saves (Delta Encoding)
- Save only changes since last save
- Reduces save file size by 90%+
- More complex implementation

## Rust Learning Opportunities

This save system will teach you:

1. **Serde Serialization** (Chapter 17 in Rust Book)
   - Derive macros
   - Custom serialization logic
   - Error handling

2. **Traits and Generics** (Chapter 10)
   - Defining traits
   - Trait bounds
   - Generic type parameters

3. **Error Handling** (Chapter 9)
   - Result<T, E> and Option<T>
   - Custom error types
   - Error propagation with ?

4. **File I/O** (Chapter 12)
   - Reading/writing files
   - Path handling
   - Directory traversal

5. **Module System** (Chapter 7)
   - Organizing code into modules
   - Public/private visibility
   - Re-exporting types

6. **Lifetimes** (Chapter 10)
   - Why textures need 'a lifetime
   - Lifetime annotations
   - Lifetime elision

## Questions & Discussion Points

1. **Save File Format**: JSON (human-readable) vs bincode (compact/fast)?
   - Recommendation: Start with JSON for debugging, switch to bincode later

2. **Autosave Frequency**: 5 minutes too frequent/infrequent?
   - Recommendation: Make configurable in settings

3. **Multiple Save Slots**: Should players have multiple save files?
   - Recommendation: Yes, support numbered slots (Save 1, Save 2, etc.)

4. **World Format**: How will chunks/terrain be represented?
   - Depends on world implementation (tile-based? voxel? 2D array?)

5. **Entity IDs**: Use sequential IDs or UUIDs?
   - Recommendation: Sequential for simplicity, UUID if networking is planned

## Summary

This save system design provides:

✅ **Extensibility**: Add new entities/stats without changing core save code
✅ **Type Safety**: Rust's type system prevents serialization bugs
✅ **Performance**: JSON for debugging, easy migration to binary
✅ **Maintainability**: Clear separation of concerns, well-documented
✅ **Future-Proof**: Versioning, migration support
✅ **Rust Learning**: Practical application of traits, serde, error handling

## Next Steps

1. **Review this plan** - Any concerns or suggestions?
2. **Create module structure** - Set up `src/save/` directory
3. **Implement SaveManager** - Core save/load functionality
4. **Add Player stats** - Health, max_health (needed for meaningful saves)
5. **Implement Saveable for Player and Slime**
6. **Integrate into main.rs** - Add save keybinding, autosave
7. **Test thoroughly** - Unit tests, integration tests, playtesting

**Estimated implementation time**: 4-6 hours for basic functionality, 2-4 hours for polish/testing.

Let me know if you want to proceed with implementation or have questions about any design decision!


## Codebase Update (Oct 2025)

Since this plan was written, the following systems have been implemented:

✅ **Stats System** (`src/stats.rs`) - Health, movement speed, attack damage, defense  
✅ **Combat System** (`src/combat.rs`) - Damage types, PlayerState (Alive/Dead)  
✅ **Collision System** (`src/collision.rs`) - AABB collision, hitboxes  
✅ **Tile/World System** (`src/tile.rs`) - WorldGrid, RenderGrid for terrain  
✅ **Player Stats** - Player now has `stats: Stats`, health, invulnerability  
✅ **Slime Health** - Slimes have health (3 HP) and is_alive flag  

**Impact on Save System:**
- Player serialization saves the existing `stats: Stats` field ✅
- Slime serialization saves `health` and `is_alive` ✅
- World serialization saves `WorldGrid` data (tiles) ✅
- PlayerState enum (Alive/Dead) handled - death state persists ✅
- Attack cooldowns and invulnerability timers reset on load ✅

---

## Adding Save Support to New Features

**IMPORTANT**: All new features with persistent state MUST implement save/load functionality.

### Quick Implementation Guide

When adding a new entity or game system, follow these steps:

#### 1. Implement the `Saveable` trait

```rust
use crate::save::{Saveable, SaveData, SaveError};
use serde::{Serialize, Deserialize};

impl Saveable for YourNewEntity<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct YourEntityData {
            // Only save persistent state (position, health, etc.)
            // Don't save: textures, animation controllers, timers
            x: i32,
            y: i32,
            health: i32,
            // ... other fields
        }

        let data = YourEntityData {
            x: self.x,
            y: self.y,
            health: self.health,
        };

        Ok(SaveData {
            data_type: "your_entity_type".to_string(),
            json_data: serde_json::to_string(&data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        #[derive(Deserialize)]
        struct YourEntityData {
            x: i32,
            y: i32,
            health: i32,
        }

        if data.data_type != "your_entity_type" {
            return Err(SaveError::CorruptedData(format!(
                "Expected your_entity_type, got {}",
                data.data_type
            )));
        }

        let entity_data: YourEntityData = serde_json::from_str(&data.json_data)?;

        // Create new instance with default resources (textures)
        let mut entity = YourNewEntity::new(
            entity_data.x,
            entity_data.y,
            AnimationController::new(), // Will be set later
        );

        // Restore saved state
        entity.health = entity_data.health;

        Ok(entity)
    }
}
```

#### 2. Update `save_game()` function (src/main.rs)

Add your entity to the entities vector:

```rust
// In save_game() function
for (i, your_entity) in your_entities.iter().enumerate() {
    let entity_save_data = your_entity.to_save_data()
        .map_err(|e| format!("Failed to save entity {}: {}", i, e))?;

    entities.push(EntitySaveData {
        entity_id: (next_id + i) as u64,
        entity_type: "your_entity_type".to_string(),
        position: (your_entity.x, your_entity.y),
        data: entity_save_data.json_data,
    });
}
```

#### 3. Update `load_game()` function (src/main.rs)

Add deserialization logic:

```rust
// In load_game() function, in the entities loop
"your_entity_type" => {
    let mut loaded_entity = YourNewEntity::from_save_data(&save_data)
        .map_err(|e| format!("Failed to load entity: {}", e))?;

    // Set up animation controller with textures
    let animation_controller = your_entity_config.create_controller(
        your_entity_texture,
        &["idle", "active"],
    )?;
    loaded_entity.set_animation_controller(animation_controller);

    your_entities.push(loaded_entity);
}
```

#### 4. Test Your Implementation

1. Create/spawn your new entity
2. Press F5 to save
3. Modify the entity's state
4. Press F9 to reload
5. Verify the entity restored to its F5 state

### What to Save vs What to Skip

**DO Save:**
- ✅ Position (x, y coordinates)
- ✅ Health/stats
- ✅ Inventory items
- ✅ Quest progress
- ✅ Persistent flags (is_alive, is_unlocked, etc.)

**DON'T Save:**
- ❌ Textures (recreate on load)
- ❌ Animation controllers (recreate on load)
- ❌ Timers (`Instant`, `Duration` - reset on load)
- ❌ Temporary state (currently jumping, attack in progress)
- ❌ Derived data (can be recalculated)

### Example: See Existing Implementations

**Player**: `src/player.rs` lines 341-479
**Slime**: `src/slime.rs` lines 187-269
**WorldGrid**: `src/tile.rs` lines 327-364

---

## Future Enhancements

The following features are designed but not yet implemented:

### Planned Features
- [ ] Autosave every 5 minutes (infrastructure ready in SaveManager)
- [ ] Multiple save slots with selection UI
- [ ] Save file browser/manager
- [ ] Cloud save sync
- [ ] Compressed save files

### Infrastructure Already in Place
- `SaveManager::should_autosave()` - Ready for autosave timing
- `SaveManager::set_save_slot()` - Ready for multi-slot support
- `SaveManager::list_saves()` - Ready for save browser UI
- `SaveManager::cleanup_autosaves()` - Ready for autosave management

