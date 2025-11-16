// Shared enums and helper structs used throughout the game

use sdl2::pixels::Color;
use std::collections::HashMap;

/// Game state enum for tracking current game mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Playing,
    ExitMenu,
    Dead, // New state for death screen
}

/// Floating text instance for tracking animated text
pub struct FloatingTextInstance {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: Color,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Debug menu state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebugMenuState {
    Closed,
    Open { selected_index: usize },
}

/// Debug menu items that can be adjusted
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebugMenuItem {
    PlayerMaxHealth,
    PlayerAttackDamage,
    PlayerAttackSpeed,
    SlimeHealth,
    SlimeContactDamage,
    ClearInventory,
}

impl DebugMenuItem {
    pub fn all() -> Vec<Self> {
        vec![
            Self::PlayerMaxHealth,
            Self::PlayerAttackDamage,
            Self::PlayerAttackSpeed,
            Self::SlimeHealth,
            Self::SlimeContactDamage,
            Self::ClearInventory,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            Self::PlayerMaxHealth => "Player Max HP",
            Self::PlayerAttackDamage => "Player Damage",
            Self::PlayerAttackSpeed => "Player Atk Spd",
            Self::SlimeHealth => "Slime Health",
            Self::SlimeContactDamage => "Slime Contact Dmg",
            Self::ClearInventory => "Clear Inventory",
        }
    }
}

/// Debug configuration for combat tuning
#[derive(Debug, Clone)]
pub struct DebugConfig {
    pub slime_base_health: i32,
    pub slime_contact_damage: f32,
}

impl DebugConfig {
    pub fn new() -> Self {
        DebugConfig {
            slime_base_health: 8,
            slime_contact_damage: 1.0,
        }
    }
}

/// Helper struct to hold all game textures
/// This avoids repeating texture parameters everywhere
pub struct GameTextures<'a> {
    pub character: &'a sdl2::render::Texture<'a>,
    pub slime: &'a sdl2::render::Texture<'a>,
    pub entity: &'a sdl2::render::Texture<'a>,
    pub punch: &'a sdl2::render::Texture<'a>,
    pub grass_tile: &'a sdl2::render::Texture<'a>,
    pub items: &'a HashMap<String, sdl2::render::Texture<'a>>,
}
