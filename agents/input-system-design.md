# AI Prompt: Input System Design

## Context

You are designing an **Input System** for Game1, a 2.5D Rust game using SDL2. The current implementation has a critical maintainability issue: all input handling is embedded in a **470-line match statement** inside the main game loop (main.rs:744-1211), making it extremely difficult to read, test, and extend.

## Project Background

**Technology Stack:**
- SDL2 for windowing and input
- Rust (learning-focused project - emphasize idiomatic patterns)
- Current input: Keyboard + Mouse
- Resolution: 640x360 logical (scaled to window)

**Existing Patterns:**
- Review `docs/patterns/entity-pattern.md` for project conventions
- Animation system uses JSON configs (see `docs/systems/animation-system.md`)
- Systems are modular (see `src/` module structure)

**Current State Analysis:**
```rust
// main.rs:740-1211 - THE PROBLEM
for event in event_pump.poll_iter() {
    match event {
        // 470 lines of deeply nested input handling
        // - Game state transitions (Playing, ExitMenu, Dead)
        // - UI interactions (inventory, debug menu, save menu)
        // - Gameplay actions (attack, item use, movement)
        // - Debug commands (F3, F5, F9, B, G keys)
        // - Mouse clicks (UI, world interaction, spawning)
        // - Tile editing with hoe tool
        // All mixed together with no clear separation
    }
}
```

**Key Pain Points:**
1. Cannot unit test input handling
2. Hard to add new keybindings without scroll hunting
3. UI state checks scattered (`is_ui_active`, inventory open, debug menu)
4. No input remapping capability
5. Keyboard state read separately for player movement (line 1214)
6. Action handling tightly coupled to main loop
7. Debug keys mixed with gameplay keys

## Your Task

Design a **modular, testable, extensible Input System** that:
1. **Decouples** input polling from action handling
2. **Separates concerns**: gameplay input, UI input, debug input
3. **Enables testing** without requiring SDL2 runtime
4. **Supports future features** like key remapping, gamepad support
5. **Maintains readability** - clear what keys do what
6. **Follows Rust idioms** - use enums, pattern matching, traits where appropriate

## Requirements

### Must Have
- [ ] Clear separation between input polling and action handling
- [ ] Support for keyboard and mouse input (SDL2 events + keyboard state)
- [ ] Different input contexts (Playing, Menu, Inventory, Debug)
- [ ] Actions as an enum (Jump, Attack, OpenInventory, etc.)
- [ ] Input mapping system (Key -> Action)
- [ ] Integration with existing game state (`GameState` enum in main.rs:58-62)
- [ ] Handle both event-based (key press) and state-based (key held) input
- [ ] Mouse position tracking and click handling (absolute coordinates)

### Should Have
- [ ] Separate "layers" for debug vs gameplay input
- [ ] Input buffering (optional - don't add if not needed yet)
- [ ] Clear error handling for invalid input states
- [ ] Easy to add new actions without touching multiple files

### Must NOT Have (Premature Features)
- ❌ Gamepad/controller support (future, but design should allow it)
- ❌ Network input prediction
- ❌ Input recording/replay
- ❌ Complex gesture recognition
- ❌ Mouse smoothing/acceleration

## Design Constraints

**Rust Learning Goals:**
This is a learning project. Your design should teach:
- Enums for finite state spaces (Actions, InputContext)
- Pattern matching for clean branching
- Traits for abstraction (InputHandler trait?)
- Ownership patterns (who owns InputState?)
- Error handling with Result/Option

**Integration Points:**
- Must work with existing SDL2 `EventPump` and `KeyboardState`
- Must respect `is_ui_active` flag (no gameplay input when UI open)
- Must integrate with existing `GameState` enum
- Player movement uses `KeyboardState::is_scancode_pressed()`
- UI systems handle their own clicks (InventoryUI, SaveExitMenu, etc.)

**Performance:**
- Input handling happens every frame (60 FPS)
- No heap allocations in hot path if possible
- Action mapping should be O(1) lookup

## Suggested Architecture Components

Consider designing (but adapt as you see fit):

1. **Action Enum** - What the player *wants* to do
   ```rust
   enum GameAction {
       Movement(Direction),
       Attack,
       OpenInventory,
       QuickSave,
       // ...
   }
   ```

2. **InputContext** - What mode the game is in
   ```rust
   enum InputContext {
       Gameplay,
       Menu,
       Inventory,
       Dead,
   }
   ```

3. **InputMapper** - Translates keys to actions
   - Configurable (could load from config file later)
   - Context-aware (same key = different action in different contexts)

4. **InputState** - Current frame's input snapshot
   - What keys are pressed/released this frame
   - Mouse position and buttons
   - Derived actions for current context

5. **Integration Layer** - How main.rs uses it
   ```rust
   // Instead of 470-line match:
   let input_state = input_system.poll(&event_pump);
   let actions = input_system.get_actions(&input_state, current_context);

   for action in actions {
       handle_action(action, &mut game_world);
   }
   ```

## Specific Problems to Solve

**Problem 1: Debug Keys Mixed with Gameplay**
Currently F3 (debug menu), F5 (save), F9 (load), B (collision boxes), G (tile grid) are in the same match as gameplay keys. How should these be separated?

**Problem 2: Modal Input (UI Active)**
When inventory/menu is open, gameplay keys should be ignored. Currently checked via `is_ui_active` in guards. How to structure this?

**Problem 3: Mouse Handling**
- UI clicks vs world clicks
- Tile editing with mouse drag (is_tilling state)
- Mouse position stored in `mouse_x`, `mouse_y` variables
- Different behavior in different contexts

**Problem 4: Keyboard State vs Events**
- Player movement uses `keyboard_state.is_scancode_pressed()` (continuous)
- Attack uses `KeyDown` event (once per press)
- How to unify these patterns?

**Problem 5: Context-Dependent Actions**
- ESC in Playing = open menu
- ESC in Menu = close menu
- ESC in Dead = open menu from death screen
- Number keys 1-9 select hotbar slots (but only if not in debug menu)

## Expected Deliverables

Provide a detailed design document including:

1. **Architecture Overview**
   - Module structure (`src/input.rs`? Split into multiple files?)
   - Key structs and enums
   - Data flow diagram (SDL2 → InputSystem → Game Logic)

2. **Action Catalog**
   - Complete list of current game actions (extracted from main.rs:744-1211)
   - Grouped by category (Movement, Combat, UI, Debug, etc.)
   - Include default keybindings

3. **API Design**
   - Public interface for InputSystem
   - How main.rs will use it (code examples)
   - How to add a new action/keybinding

4. **Implementation Plan**
   - Phase 1: Core action enum and mapper (minimal, just enough to replace match statement)
   - Phase 2: Context handling and input layers
   - Phase 3: Polish and configuration (optional)

5. **Migration Strategy**
   - How to incrementally refactor the 470-line match
   - Which parts to extract first (lowest risk, highest value)
   - Testing strategy at each step

6. **Rust Patterns**
   - Which traits to use (if any)
   - Ownership model (who owns what)
   - Error handling approach
   - Why these choices? (explain for learning)

## Success Criteria

Your design is successful if:
- ✅ The 470-line match statement can be eliminated
- ✅ Input handling is unit testable
- ✅ Adding a new keybinding takes < 5 lines of code
- ✅ Context switching (Gameplay ↔ Menu) is explicit and clear
- ✅ Debug input is separated from gameplay input
- ✅ Code is more readable than current implementation
- ✅ Uses idiomatic Rust patterns
- ✅ No performance regression (input still processes in < 1ms)
- ✅ Future gamepad support is architecturally feasible

## Anti-Patterns to Avoid

- ❌ Don't create an overly generic input system (no supporting 20 different input backends)
- ❌ Don't add configuration files if hardcoded mappings work for now
- ❌ Don't abstract away SDL2 if it's the only backend we'll use
- ❌ Don't create a giant InputHandler god-object
- ❌ Don't use dynamic dispatch (trait objects) unless absolutely necessary
- ❌ Don't make the API so complex that it's harder to use than the match statement

## References

Study these files for context:
- `src/main.rs:728-1211` - Current input handling (the beast we're taming)
- `src/main.rs:58-62` - GameState enum
- `src/gui/inventory_ui.rs` - Example of UI handling its own input
- `docs/patterns/entity-pattern.md` - Project conventions
- `docs/systems/animation-system.md` - Example of config-driven system

## Questions to Answer

As you design, explicitly address:
1. Should input mapping be data-driven (JSON/TOML) or code-defined?
2. How to handle modifier keys (Shift+Left vs Left)?
3. Should InputSystem own the EventPump, or just process events?
4. How to mock/fake input for testing?
5. Where does "action validation" live (e.g., can't attack if dead)?

## Final Note

Remember: This is a **learning project**. Your design should be:
- **Simple enough** to implement in a reasonable timeframe
- **Idiomatic enough** to teach good Rust patterns
- **Extensible enough** to support future features
- **Well-documented enough** to understand 6 months from now

Prioritize **clarity** over **cleverness**. If you must choose between a clever abstraction and straightforward code, choose straightforward.

Good luck! 🎮
