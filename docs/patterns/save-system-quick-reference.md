# Save System Quick Reference

## For Developers: Adding Save Support to New Entities

### Step-by-Step Checklist

- [ ] 1. Implement `Saveable` trait in your entity file
- [ ] 2. Add entity serialization to `save_game()` in main.rs
- [ ] 3. Add entity deserialization to `load_game()` in main.rs
- [ ] 4. Test: Save → Modify → Load → Verify

### Code Template

```rust
// In your entity file (e.g., src/your_entity.rs)
use crate::save::{Saveable, SaveData, SaveError};
use serde::{Serialize, Deserialize};

impl Saveable for YourEntity<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        #[derive(Serialize)]
        struct YourEntityData {
            x: i32,
            y: i32,
            // Add all persistent fields here
        }

        let data = YourEntityData {
            x: self.x,
            y: self.y,
        };

        Ok(SaveData {
            data_type: "your_entity".to_string(),
            json_data: serde_json::to_string(&data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> {
        #[derive(Deserialize)]
        struct YourEntityData {
            x: i32,
            y: i32,
        }

        if data.data_type != "your_entity" {
            return Err(SaveError::CorruptedData(format!(
                "Expected your_entity, got {}", data.data_type
            )));
        }

        let entity_data: YourEntityData = serde_json::from_str(&data.json_data)?;

        let mut entity = YourEntity::new(
            entity_data.x,
            entity_data.y,
            AnimationController::new(),
        );

        Ok(entity)
    }
}

// Add this method to allow setting animation after load
pub fn set_animation_controller(&mut self, controller: AnimationController<'a>) {
    self.animation_controller = controller;
}
```

### What to Save

✅ **DO Save:**
- Position (x, y)
- Health/stats
- Inventory
- Quest flags
- Persistent state

❌ **DON'T Save:**
- Textures
- Animation controllers
- Timers (Instant/Duration)
- Temporary state
- Derived values

### Existing Examples

Look at these files for reference:
- **Player**: `src/player.rs:341-479`
- **Slime**: `src/slime.rs:187-269`
- **WorldGrid**: `src/tile.rs:327-364`

## For Players: Using the Save System

### Controls
- **F5** - Quick save (instant, no menu)
- **F9** - Load from save
- **ESC** - Exit menu
  - "Save and Exit" - Saves and quits
  - "Cancel" - Return to game

### Save Location
- **macOS/Linux**: `~/.game1/saves/slot_1.json`
- **Windows**: `%USERPROFILE%\.game1\saves\slot_1.json`

### Auto Features
- Game automatically loads your save when you start
- All progress is saved (player stats, enemies, world tiles)
- Save file is human-readable JSON

### Tips
- Press F5 before attempting difficult sections
- F9 lets you retry without closing the game
- Exit menu always saves your progress
