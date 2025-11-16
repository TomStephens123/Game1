// Game module - Contains all game logic and state management
//
// This module contains:
// - world.rs: GameWorld struct and entity management
// - systems.rs: Systems configuration and helper systems
// - types.rs: Shared enums and helper structs
// - ui_manager.rs: UI management struct
// - constructors.rs: Game initialization (new/load)
// - events.rs: Input handling and event processing
// - update.rs: Game logic and physics
// - rendering.rs: Drawing and visual rendering

// Module declarations
pub mod world;
pub mod systems;
pub mod types;
pub mod ui_manager;
pub mod constructors;
pub mod events;
pub mod update;
pub mod rendering;

// Re-export types for convenience
pub use types::*;
pub use world::GameWorld;
