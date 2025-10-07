# Screen-Space Menu System

**Status**: 📋 PLANNED

## Overview

This document outlines the implementation plan for Game1's screen-space GUI menus, specifically:
1. **Save/Exit Menu** - Refactor existing implementation to use new UI system architecture
2. **Death/Respawn Screen** - New system for player death with respawn timer

Both menus follow the Screen-Space GUI pattern defined in `docs/ui-system.md`.

---

## Architecture Reference

**Parent Document**: `docs/ui-system.md` - System 2: Screen-Space GUI

**Key Characteristics**:
- Fixed screen coordinates (not attached to entities)
- Renders on top layer (above world-space HUD)
- Stateful (open/closed, has content)
- Uses SDL2 primitives (procedural rendering)

**Rendering Order**:
```
1. World background (tiles)
2. Entities (player, enemies)
3. Visual effects
4. World-Space HUD (health bars)
5. Screen-Space GUI (menus) ← This system
```

---

## Current State Analysis

### Existing Save/Exit Menu

**Location**: `src/main.rs:398-492` (`render_exit_menu()`)

**Current Implementation**:
- ✅ Semi-transparent overlay (RGBA 0,0,0,180)
- ✅ Centered menu box (500x240px)
- ✅ Double border styling
- ✅ Keyboard navigation (Up/Down/Enter)
- ✅ Two options: "Save and Exit", "Cancel"
- ✅ Selection highlighting
- ⚠️ Tightly coupled to main.rs (not modular)
- ⚠️ Uses raw SDL2 calls (no abstraction)

**State Management**:
```rust
enum GameState {
    Playing,
    ExitMenu,
}

enum ExitMenuOption {
    SaveAndExit,
    Cancel,
}
```

**Integration Points**:
- Triggered by ESC key
- Blocks game updates when open
- Calls `save_game()` on confirmation
- Returns to Playing state on cancel

### Debug Menu (Reference Implementation)

**Location**: `src/main.rs:495-651` (`render_debug_menu()`)

**Similar patterns**:
- Semi-transparent overlay
- Centered menu box (400x280px)
- Item selection with visual feedback
- Keyboard navigation (Arrow keys, Shift modifier)
- Toggled by F3 key

**State Management**:
```rust
enum DebugMenuState {
    Closed,
    Open { selected_index: usize },
}
```

---

## Implementation Plan

### Phase 1: Refactor Save/Exit Menu

**Goal**: Move Save/Exit menu into modular `src/gui/` module following screen-space GUI architecture

#### Step 1.1: Create GUI Module

**File**: `src/gui/mod.rs`
```rust
//! Screen-Space GUI System
//!
//! This module provides UI elements that render at fixed screen positions,
//! independent of world entities. See `docs/ui-system.md` System 2 for architecture.

pub mod menu;

pub use menu::{Menu, MenuStyle, MenuItem};
```

#### Step 1.2: Create Base Menu Component

**File**: `src/gui/menu.rs`

**Purpose**: Reusable menu component for all overlay menus (save/exit, pause, death, etc.)

**Design**:
```rust
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Configuration for menu appearance
#[derive(Debug, Clone)]
pub struct MenuStyle {
    /// Menu box width in pixels
    pub width: u32,

    /// Menu box height in pixels
    pub height: u32,

    /// Background color
    pub background_color: Color,

    /// Border color
    pub border_color: Color,

    /// Border thickness (draws double border)
    pub border_thickness: u32,

    /// Overlay darkness (0-255, higher = darker)
    pub overlay_alpha: u8,

    /// Title text color
    pub title_color: Color,

    /// Normal item text color
    pub item_color: Color,

    /// Selected item text color
    pub selected_item_color: Color,

    /// Selection highlight color
    pub highlight_color: Color,
}

impl Default for MenuStyle {
    fn default() -> Self {
        MenuStyle {
            width: 500,
            height: 240,
            background_color: Color::RGB(30, 30, 40),
            border_color: Color::RGB(100, 100, 120),
            border_thickness: 2,
            overlay_alpha: 180,
            title_color: Color::RGB(220, 220, 240),
            item_color: Color::RGB(160, 160, 170),
            selected_item_color: Color::RGB(255, 255, 255),
            highlight_color: Color::RGB(80, 100, 140),
        }
    }
}

/// A menu item with text and optional value display
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub text: String,
    pub enabled: bool,  // For future "disabled" option styling
}

/// A stateful overlay menu component
pub struct Menu {
    title: String,
    items: Vec<MenuItem>,
    selected_index: usize,
    style: MenuStyle,
}

impl Menu {
    /// Creates a new menu with default styling
    pub fn new(title: String, items: Vec<MenuItem>) -> Self {
        Menu {
            title,
            items,
            selected_index: 0,
            style: MenuStyle::default(),
        }
    }

    /// Creates a menu with custom styling
    pub fn with_style(title: String, items: Vec<MenuItem>, style: MenuStyle) -> Self {
        Menu {
            title,
            items,
            selected_index: 0,
            style,
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_index == 0 {
            self.selected_index = self.items.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.items.len();
    }

    /// Get currently selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Render the menu at screen center
    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // 1. Semi-transparent overlay (darken screen)
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(0, 0, 0, self.style.overlay_alpha));
        canvas.fill_rect(None)?;
        canvas.set_blend_mode(sdl2::render::BlendMode::None);

        // 2. Calculate centered position
        let (screen_width, screen_height) = canvas.output_size()?;
        let menu_x = (screen_width - self.style.width) / 2;
        let menu_y = (screen_height - self.style.height) / 2;

        // 3. Menu background
        canvas.set_draw_color(self.style.background_color);
        canvas.fill_rect(Rect::new(
            menu_x as i32,
            menu_y as i32,
            self.style.width,
            self.style.height,
        ))?;

        // 4. Double border
        canvas.set_draw_color(self.style.border_color);
        canvas.draw_rect(Rect::new(
            menu_x as i32,
            menu_y as i32,
            self.style.width,
            self.style.height,
        ))?;
        canvas.draw_rect(Rect::new(
            (menu_x + 2) as i32,
            (menu_y + 2) as i32,
            self.style.width - 4,
            self.style.height - 4,
        ))?;

        // 5. Title (centered)
        draw_simple_text(
            canvas,
            &self.title,
            (menu_x + self.style.width / 2 - (self.title.len() as u32 * 8)) as i32,
            (menu_y + 30) as i32,
            self.style.title_color,
            3,
        )?;

        // 6. Menu items
        let item_height = 60;
        let item_start_y = menu_y + 100;

        for (i, item) in self.items.iter().enumerate() {
            let item_y = item_start_y + (i as u32 * item_height);
            let is_selected = i == self.selected_index;

            // Selection highlight
            if is_selected {
                canvas.set_draw_color(self.style.highlight_color);
                canvas.fill_rect(Rect::new(
                    (menu_x + 15) as i32,
                    item_y as i32 - 3,
                    self.style.width - 30,
                    36,
                ))?;
            }

            // Item text
            let text_color = if is_selected {
                self.style.selected_item_color
            } else {
                self.style.item_color
            };

            draw_simple_text(
                canvas,
                &item.text,
                (menu_x + 80) as i32,
                item_y as i32,
                text_color,
                3,
            )?;
        }

        Ok(())
    }
}

// Helper function (will be imported from main.rs or moved to shared module)
fn draw_simple_text(
    canvas: &mut Canvas<Window>,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
    scale: u32,
) -> Result<(), String> {
    // Implementation will be moved from main.rs or imported
    // For now, this is a placeholder signature
    unimplemented!("Text rendering needs to be refactored from main.rs")
}
```

**Key Design Decisions**:
- **Stateful component** (unlike world-space HUD components which are stateless)
- **Owns menu state** (selected_index, items)
- **Procedural rendering** (SDL2 primitives, no sprites)
- **Reusable** (can create multiple menu types with different styles)

#### Step 1.3: Create Save/Exit Menu Wrapper

**File**: `src/gui/save_exit_menu.rs`

```rust
use super::{Menu, MenuItem};

/// Options in the save/exit menu
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveExitOption {
    SaveAndExit,
    Cancel,
}

/// State of the save/exit menu
pub struct SaveExitMenu {
    menu: Menu,
}

impl SaveExitMenu {
    pub fn new() -> Self {
        let items = vec![
            MenuItem {
                text: "SAVE AND EXIT".to_string(),
                enabled: true,
            },
            MenuItem {
                text: "CANCEL".to_string(),
                enabled: true,
            },
        ];

        SaveExitMenu {
            menu: Menu::new("EXIT".to_string(), items),
        }
    }

    /// Navigate up
    pub fn navigate_up(&mut self) {
        self.menu.select_previous();
    }

    /// Navigate down
    pub fn navigate_down(&mut self) {
        self.menu.select_next();
    }

    /// Get selected option
    pub fn selected_option(&self) -> SaveExitOption {
        match self.menu.selected_index() {
            0 => SaveExitOption::SaveAndExit,
            1 => SaveExitOption::Cancel,
            _ => SaveExitOption::Cancel,
        }
    }

    /// Render the menu
    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        self.menu.render(canvas)
    }
}
```

#### Step 1.4: Refactor Text Rendering

**Problem**: `draw_simple_text()` is currently defined in `main.rs`

**Solution**: Create shared text module

**File**: `src/text.rs`
```rust
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Renders bitmap text using procedural rectangles
pub fn draw_simple_text(
    canvas: &mut Canvas<Window>,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
    scale: u32,
) -> Result<(), String> {
    // Move implementation from main.rs
    // (existing implementation remains unchanged)
}
```

**Integration**:
- Export in `src/lib.rs`: `pub mod text;`
- Import in main.rs: `use crate::text::draw_simple_text;`
- Import in gui module: `use crate::text::draw_simple_text;`

#### Step 1.5: Update main.rs Integration

**Changes**:
1. Add module: `mod gui;`
2. Import menu: `use gui::{SaveExitMenu, SaveExitOption};`
3. Replace state management:
   ```rust
   // OLD:
   let mut game_state = GameState::Playing;
   let mut exit_menu_selection = ExitMenuOption::SaveAndExit;

   // NEW:
   let mut game_state = GameState::Playing;
   let mut save_exit_menu = SaveExitMenu::new();
   ```
4. Update event handling:
   ```rust
   Event::KeyDown { keycode: Some(Keycode::Up), .. }
       if game_state == GameState::ExitMenu => {
       save_exit_menu.navigate_up();
   }

   Event::KeyDown { keycode: Some(Keycode::Down), .. }
       if game_state == GameState::ExitMenu => {
       save_exit_menu.navigate_down();
   }

   Event::KeyDown { keycode: Some(Keycode::Return | Keycode::Space), .. }
       if game_state == GameState::ExitMenu => {
       match save_exit_menu.selected_option() {
           SaveExitOption::SaveAndExit => {
               if let Err(e) = save_game(&mut save_manager, &player, &slimes, &world_grid) {
                   eprintln!("Failed to save: {}", e);
               }
               break 'running;
           }
           SaveExitOption::Cancel => {
               game_state = GameState::Playing;
           }
       }
   }
   ```
5. Update rendering:
   ```rust
   // Render exit menu if active
   if game_state == GameState::ExitMenu {
       save_exit_menu.render(&mut canvas)?;
   }
   ```
6. **Remove old code**:
   - Delete `render_exit_menu()` function
   - Delete `ExitMenuOption` enum (moved to gui module)

---

### Phase 2: Implement Death/Respawn Screen

**Goal**: Create death screen with respawn timer

#### Step 2.1: Design Death Screen

**Visual Design**:
- Dark overlay (more opaque than save menu - alpha 220)
- Large "YOU DIED" text (centered, red)
- Respawn timer countdown (e.g., "Respawning in 3...")
- Smaller instructions: "ESC to exit"
- No player input during countdown (auto-respawn)

**Respawn Behavior**:
- Countdown: 3 seconds
- Auto-respawn at world center: (GAME_WIDTH / 2, GAME_HEIGHT / 2)
- Restore full health
- Clear all status effects (invulnerability resets)
- Optional: penalty system (lose XP, gold, etc.) - future feature

#### Step 2.2: Add Death State to Player

**Current State**: Player already has `PlayerState` enum in `src/combat.rs`

**Verify Current Implementation**:
```rust
// src/combat.rs
pub enum PlayerState {
    Alive,
    Dead,
}

impl PlayerState {
    pub fn is_alive(&self) -> bool {
        matches!(self, PlayerState::Alive)
    }

    pub fn is_dead(&self) -> bool {
        matches!(self, PlayerState::Dead)
    }
}
```

**Already integrated** ✅ - Player has `pub state: PlayerState` field

**Death Trigger**: Already implemented in `Player::take_damage()`:
```rust
if self.stats.health.is_depleted() {
    self.state = PlayerState::Dead;
}
```

#### Step 2.3: Create Death Screen Component

**File**: `src/gui/death_screen.rs`

```rust
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::{Duration, Instant};
use crate::text::draw_simple_text;

/// Configuration for death screen
#[derive(Debug, Clone)]
pub struct DeathScreenStyle {
    /// Overlay darkness (0-255, higher = darker)
    pub overlay_alpha: u8,

    /// "YOU DIED" text color
    pub title_color: Color,

    /// Respawn timer text color
    pub timer_color: Color,

    /// Instruction text color
    pub instruction_color: Color,
}

impl Default for DeathScreenStyle {
    fn default() -> Self {
        DeathScreenStyle {
            overlay_alpha: 220,  // Darker than normal menus
            title_color: Color::RGB(255, 50, 50),     // Red
            timer_color: Color::RGB(255, 255, 100),   // Yellow
            instruction_color: Color::RGB(150, 150, 160),  // Gray
        }
    }
}

/// State of the death screen
pub struct DeathScreen {
    respawn_duration: Duration,
    death_time: Option<Instant>,
    style: DeathScreenStyle,
}

impl DeathScreen {
    /// Creates a new death screen with 3-second respawn timer
    pub fn new() -> Self {
        DeathScreen {
            respawn_duration: Duration::from_secs(3),
            death_time: None,
            style: DeathScreenStyle::default(),
        }
    }

    /// Creates death screen with custom respawn duration
    pub fn with_duration(duration: Duration) -> Self {
        DeathScreen {
            respawn_duration: duration,
            death_time: None,
            style: DeathScreenStyle::default(),
        }
    }

    /// Trigger death screen (start timer)
    pub fn trigger(&mut self) {
        self.death_time = Some(Instant::now());
    }

    /// Reset death screen (clear timer)
    pub fn reset(&mut self) {
        self.death_time = None;
    }

    /// Check if respawn timer has expired
    pub fn should_respawn(&self) -> bool {
        if let Some(death_time) = self.death_time {
            death_time.elapsed() >= self.respawn_duration
        } else {
            false
        }
    }

    /// Get remaining respawn time in seconds
    pub fn remaining_time(&self) -> f32 {
        if let Some(death_time) = self.death_time {
            let elapsed = death_time.elapsed().as_secs_f32();
            let total = self.respawn_duration.as_secs_f32();
            (total - elapsed).max(0.0)
        } else {
            0.0
        }
    }

    /// Render death screen overlay
    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        if self.death_time.is_none() {
            return Ok(());
        }

        // Dark overlay
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(0, 0, 0, self.style.overlay_alpha));
        canvas.fill_rect(None)?;
        canvas.set_blend_mode(sdl2::render::BlendMode::None);

        let (screen_width, screen_height) = canvas.output_size()?;
        let center_x = screen_width / 2;
        let center_y = screen_height / 2;

        // "YOU DIED" text (large, centered)
        draw_simple_text(
            canvas,
            "YOU DIED",
            (center_x - 100) as i32,  // Rough centering
            (center_y - 60) as i32,
            self.style.title_color,
            4,  // Large scale
        )?;

        // Respawn timer (medium, centered below title)
        let remaining = self.remaining_time();
        if remaining > 0.0 {
            let timer_text = format!("Respawning in {:.0}...", remaining.ceil());
            draw_simple_text(
                canvas,
                &timer_text,
                (center_x - 120) as i32,
                (center_y + 10) as i32,
                self.style.timer_color,
                2,
            )?;
        }

        // Instructions (small, bottom center)
        draw_simple_text(
            canvas,
            "ESC to exit",
            (center_x - 60) as i32,
            (center_y + 80) as i32,
            self.style.instruction_color,
            1,
        )?;

        Ok(())
    }
}

impl Default for DeathScreen {
    fn default() -> Self {
        Self::new()
    }
}
```

**Key Features**:
- Timer-based (auto-respawn after duration)
- Stateful (tracks death time)
- Visual feedback (countdown display)
- Can check `should_respawn()` to trigger respawn logic

#### Step 2.4: Add Respawn Logic to Player

**File**: `src/player.rs`

**Add method**:
```rust
impl<'a> Player<'a> {
    /// Respawn player at a position with full health
    pub fn respawn(&mut self, x: i32, y: i32) {
        // Restore health
        self.stats.health.restore_full();

        // Reset state
        self.state = PlayerState::Alive;

        // Clear combat state
        self.is_attacking = false;
        self.is_taking_damage = false;

        // Reset invulnerability
        self.is_invulnerable = false;
        self.invulnerability_timer = Instant::now();

        // Reset position
        self.x = x;
        self.y = y;
        self.velocity_x = 0;
        self.velocity_y = 0;

        println!("Player respawned at ({}, {}) with full health", x, y);
    }
}
```

#### Step 2.5: Update GameState Enum

**File**: `src/main.rs`

**Add new state**:
```rust
#[derive(Debug, Clone, PartialEq)]
enum GameState {
    Playing,
    ExitMenu,
    Dead,  // New state for death screen
}
```

#### Step 2.6: Integrate Death Screen in main.rs

**Changes**:

1. **Import**:
   ```rust
   use gui::DeathScreen;
   ```

2. **Initialize**:
   ```rust
   // After SaveExitMenu initialization
   let mut death_screen = DeathScreen::new();
   ```

3. **Death Detection** (in game loop update section):
   ```rust
   // Only process game updates when playing
   if game_state == GameState::Playing {
       // Check for player death
       if player.state.is_dead() && game_state != GameState::Dead {
           game_state = GameState::Dead;
           death_screen.trigger();
           println!("Player died!");
       }

       // ... rest of game updates ...
   }
   ```

4. **Respawn Check** (after update section):
   ```rust
   // Check for respawn
   if game_state == GameState::Dead {
       if death_screen.should_respawn() {
           // Respawn at world center
           player.respawn(GAME_WIDTH as i32 / 2, GAME_HEIGHT as i32 / 2);
           death_screen.reset();
           game_state = GameState::Playing;
           println!("Player respawned!");
       }
   }
   ```

5. **ESC key handling during death** (in event loop):
   ```rust
   Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
       match game_state {
           GameState::Playing => {
               // Open exit menu
               game_state = GameState::ExitMenu;
           }
           GameState::ExitMenu => {
               // Close menu (Cancel)
               game_state = GameState::Playing;
           }
           GameState::Dead => {
               // Allow exit during death
               game_state = GameState::ExitMenu;
               death_screen.reset();
           }
       }
   }
   ```

6. **Rendering** (in render section):
   ```rust
   // Render death screen if dead
   if game_state == GameState::Dead {
       death_screen.render(&mut canvas)?;
   }

   // Render exit menu if active (can be shown over death screen)
   if game_state == GameState::ExitMenu {
       save_exit_menu.render(&mut canvas)?;
   }
   ```

7. **Block input during death**:
   ```rust
   // Movement input handling
   if game_state == GameState::Playing && player.state.is_alive() {
       // ... movement code ...
   }

   // Attack input handling
   Event::KeyDown { keycode: Some(Keycode::Space), .. }
       if game_state == GameState::Playing && player.state.is_alive() => {
       // ... attack code ...
   }
   ```

---

## Testing Plan

### Phase 1 Testing (Save/Exit Menu Refactor)

**Verification Steps**:
1. ✅ Press ESC → Save/Exit menu appears
2. ✅ Press Up/Down → Selection changes
3. ✅ Press Enter on "Save and Exit" → Game saves and exits
4. ✅ Press Enter on "Cancel" → Menu closes, game continues
5. ✅ Press ESC in menu → Menu closes (cancel behavior)
6. ✅ Verify menu appearance matches old implementation

**Success Criteria**:
- No functional changes (behavior identical to old implementation)
- Code is modular (menu logic in `src/gui/`)
- Text rendering is shared (not duplicated)

### Phase 2 Testing (Death Screen)

**Verification Steps**:
1. ✅ Reduce player health to 0 → Death screen appears
2. ✅ Countdown timer shows "Respawning in 3...", "2...", "1..."
3. ✅ After 3 seconds → Player respawns at center with full health
4. ✅ During countdown, press ESC → Exit menu appears, death screen cleared
5. ✅ During countdown, movement/attack inputs do nothing
6. ✅ After respawn, verify player can move/attack normally

**Edge Cases**:
- Death while exit menu is open
- Multiple deaths in quick succession
- ESC during death → exit → cancel (should return to Playing, not Dead)

**Success Criteria**:
- Death screen triggers automatically on health depletion
- Respawn timer functions correctly (3 seconds)
- Player respawns at correct position with full health
- No input leaks during death state

---

## File Structure After Implementation

```
src/
├── gui/                     # NEW: Screen-Space GUI module
│   ├── mod.rs              # Module exports
│   ├── menu.rs             # Base Menu component
│   ├── save_exit_menu.rs   # Save/Exit menu wrapper
│   └── death_screen.rs     # Death/Respawn screen
├── text.rs                 # NEW: Shared text rendering
├── main.rs                 # MODIFIED: Integrate GUI components
└── player.rs               # MODIFIED: Add respawn() method

docs/
└── features/
    └── screen-space-menus.md  # This document
```

---

## Future Enhancements

### Menu System Extensions
- **Pause Menu**: Settings, resume, exit options
- **Settings Menu**: Volume, controls, display settings
- **Inventory Screen**: Item management
- **Character Sheet**: Stats display

### Death System Extensions
- **Death Penalty**: Lose gold, XP, or items on death
- **Multiple Lives**: Life counter system
- **Permadeath Mode**: Game over screen instead of respawn
- **Respawn Choice**: Choose respawn location (checkpoints)

### UI Library Integration (Long-term)
- Replace custom menu system with `egui` for advanced GUI
- Add mouse support for menu navigation
- Implement keyboard/gamepad input remapping
- Add accessibility features (larger text, colorblind modes)

---

## Dependencies

**Required Before Implementation**:
- ✅ UI System Architecture (`docs/ui-system.md`) - Already defined
- ✅ Player death detection (`PlayerState::Dead`) - Already implemented
- ✅ Stats system with health (`Stats`, `Health`) - Already implemented
- ✅ Text rendering (`draw_simple_text()`) - Already implemented in main.rs

**No New External Dependencies Required** - Uses existing SDL2 primitives

---

## Appendix: Design Rationale

### Why Refactor Save/Exit Menu?

**Current Issues**:
- Tightly coupled to main.rs (600+ line function)
- Not reusable (can't easily create other menus)
- Text rendering duplicated in debug menu

**Benefits of Refactoring**:
- Modular architecture (follows `docs/ui-system.md`)
- Reusable components (death screen reuses Menu base)
- Easier to extend (pause menu, settings menu, etc.)
- Clear separation of concerns (GUI code in `src/gui/`)

### Why Auto-Respawn vs Manual?

**Decision**: Auto-respawn after 3 seconds

**Rationale**:
- ✅ Reduces friction (player doesn't need to press button)
- ✅ Gives moment to process death (countdown provides feedback)
- ✅ Simpler implementation (no button state tracking)
- ✅ Common pattern in action games

**Alternative** (manual respawn):
- "Press SPACE to respawn" text
- Requires additional input handling
- May feel slower/more tedious

**Future Option**: Make respawn mode configurable:
```rust
pub enum RespawnMode {
    Auto(Duration),      // Auto-respawn after timer
    Manual,              // Press button to respawn
    Checkpoint(String),  // Choose respawn location
}
```

### Why Separate DeathScreen from Menu?

**Design Choice**: `DeathScreen` is not a subclass/variant of `Menu`

**Rationale**:
- Different interaction model (timer-based vs selection-based)
- Different state (Instant vs selected_index)
- Different visual layout (centered text vs menu items)
- Composition over inheritance (both can reuse text rendering)

**Future Refactoring**: Could extract common "overlay" component:
```rust
pub struct OverlayScreen {
    overlay_alpha: u8,
    content: Box<dyn OverlayContent>,
}

trait OverlayContent {
    fn render(&self, canvas: &mut Canvas, center: (u32, u32)) -> Result<(), String>;
}
```
