use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

mod animation;
mod attack_effect;
mod collision;
mod combat;
mod dropped_item;
mod gui;
mod inventory;
mod item;
mod player;
mod render;
mod save;
mod slime;
mod sprite;
mod stats;
mod text;
mod the_entity;
mod tile;
mod ui;

use animation::{AnimationConfig, AnimationController};
use attack_effect::AttackEffect;
use collision::{
    calculate_overlap, check_collisions_with_collection, check_static_collisions, Collidable,
    StaticCollidable, StaticObject,
};
use combat::{DamageEvent, DamageSource};
use dropped_item::DroppedItem;
use gui::{SaveExitMenu, SaveExitOption, DeathScreen, InventoryUI};
use inventory::PlayerInventory;
use item::{ItemRegistry, ItemProperties, ToolType};
use player::Player;
use render::render_with_depth_sorting;
use save::{SaveManager, SaveFile, SaveMetadata, SaveType, WorldSaveData, EntitySaveData, Saveable, SaveData, CURRENT_SAVE_VERSION};
use slime::Slime;
use sprite::SpriteSheet;
use stats::{ModifierEffect, StatModifier, StatType};
use text::draw_simple_text;
use the_entity::{TheEntity, EntityState, EntityType};
use tile::{TileId, WorldGrid, RenderGrid};
use ui::{HealthBar, HealthBarStyle, FloatingText, BuffDisplay};
use std::time::{SystemTime, Instant};
use serde::Deserialize;
use std::collections::HashMap;

// Game resolution constants
const GAME_WIDTH: u32 = 640;
const GAME_HEIGHT: u32 = 360;
const SPRITE_SCALE: u32 = 2;

/// Game state for menu/gameplay tracking
#[derive(Debug, Clone, PartialEq)]
enum GameState {
    Playing,
    ExitMenu,
    Dead, // New state for death screen
}

/// Floating text instance for tracking animated text
pub struct FloatingTextInstance {
    x: f32,
    y: f32,
    text: String,
    color: Color,
    lifetime: f32,
    max_lifetime: f32,
}

/// Debug menu state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebugMenuState {
    Closed,
    Open { selected_index: usize },
}

/// Debug menu items that can be adjusted
#[derive(Debug, Clone, Copy, PartialEq)]
enum DebugMenuItem {
    PlayerMaxHealth,
    PlayerAttackDamage,
    PlayerAttackSpeed,
    SlimeHealth,
    SlimeContactDamage,
    ClearInventory,
}

impl DebugMenuItem {
    fn all() -> Vec<Self> {
        vec![
            Self::PlayerMaxHealth,
            Self::PlayerAttackDamage,
            Self::PlayerAttackSpeed,
            Self::SlimeHealth,
            Self::SlimeContactDamage,
            Self::ClearInventory,
        ]
    }

    fn name(&self) -> &str {
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
    slime_base_health: i32,
    slime_contact_damage: f32,
}

impl DebugConfig {
    fn new() -> Self {
        DebugConfig {
            slime_base_health: 8,
            slime_contact_damage: 1.0,
        }
    }
}

/// GameWorld encapsulates all game entities and world state
/// This struct owns all the game objects that exist in the world
pub struct GameWorld<'a> {
    pub player: Player<'a>,
    pub slimes: Vec<Slime<'a>>,
    pub entities: Vec<TheEntity<'a>>,
    pub dropped_items: Vec<DroppedItem<'a>>,
    pub world_grid: WorldGrid,
    pub render_grid: RenderGrid,
    pub player_inventory: PlayerInventory,
    pub attack_effects: Vec<AttackEffect<'a>>,
    pub floating_texts: Vec<FloatingTextInstance>,
    pub active_attack: Option<combat::AttackEvent>,
}

/// Systems holds configuration data and helper systems
/// This struct contains things that configure gameplay but aren't entities
pub struct Systems {
    pub player_config: AnimationConfig,
    pub slime_config: AnimationConfig,
    pub punch_config: AnimationConfig,
    pub debug_config: DebugConfig,
    pub static_objects: Vec<StaticObject>,
    pub regen_timer: Instant,
    pub regen_interval: f32,
    pub has_regen: bool,
}

impl Systems {
    /// Create systems with default configuration
    pub fn new(
        player_config: AnimationConfig,
        slime_config: AnimationConfig,
        punch_config: AnimationConfig,
    ) -> Self {
        let boundary_thickness = 10;
        let static_objects = vec![
            StaticObject::new(0, -(boundary_thickness as i32), GAME_WIDTH, boundary_thickness),
            StaticObject::new(-(boundary_thickness as i32), 0, boundary_thickness, GAME_HEIGHT),
            StaticObject::new(GAME_WIDTH as i32, 0, boundary_thickness, GAME_HEIGHT),
            StaticObject::new(0, GAME_HEIGHT as i32, GAME_WIDTH, boundary_thickness),
        ];

        Systems {
            player_config,
            slime_config,
            punch_config,
            debug_config: DebugConfig::new(),
            static_objects,
            regen_timer: Instant::now(),
            regen_interval: 5.0,
            has_regen: false,
        }
    }
}

/// UIManager holds all UI state and components
/// This struct manages menus, HUD elements, and debug overlays
pub struct UIManager<'a> {
    pub save_exit_menu: SaveExitMenu,
    pub death_screen: DeathScreen,
    pub inventory_ui: InventoryUI<'a>,
    pub player_health_bar: HealthBar,
    pub enemy_health_bar: HealthBar,
    pub floating_text_renderer: FloatingText,
    pub buff_display: BuffDisplay<'a>,
    pub debug_menu_state: DebugMenuState,
    pub show_collision_boxes: bool,
    pub show_tile_grid: bool,
    pub is_tilling: bool,
    pub last_tilled_tile: Option<(i32, i32)>,
    pub mouse_x: i32,
    pub mouse_y: i32,
}

/// Handle all input events from SDL2 event pump
/// Returns true if the game should quit
fn handle_events<'a>(
    event_pump: &mut sdl2::EventPump,
    game_state: &mut GameState,
    world: &mut GameWorld<'a>,
    systems: &mut Systems,
    ui: &mut UIManager<'a>,
    save_manager: &mut SaveManager,
    character_texture: &'a sdl2::render::Texture<'a>,
    slime_texture: &'a sdl2::render::Texture<'a>,
    entity_texture: &'a sdl2::render::Texture<'a>,
    punch_texture: &'a sdl2::render::Texture<'a>,
    item_textures: &'a HashMap<String, sdl2::render::Texture<'a>>,
    item_registry: &ItemRegistry,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
) -> Result<bool, String> {
    let is_ui_active = ui.inventory_ui.is_open ||
                       matches!(ui.debug_menu_state, DebugMenuState::Open { .. }) ||
                       *game_state == GameState::ExitMenu ||
                       *game_state == GameState::Dead;

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => return Ok(true),
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                match *game_state {
                    GameState::Playing => {
                        // Close other UIs first before opening save/exit menu
                        if ui.inventory_ui.is_open {
                            ui.inventory_ui.is_open = false;
                        } else if matches!(ui.debug_menu_state, DebugMenuState::Open { .. }) {
                            ui.debug_menu_state = DebugMenuState::Closed;
                        } else {
                            *game_state = GameState::ExitMenu;
                        }
                    }
                    GameState::ExitMenu => {
                        *game_state = GameState::Playing;
                    }
                    GameState::Dead => {
                        *game_state = GameState::ExitMenu;
                        ui.death_screen.reset();
                    }
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } if *game_state == GameState::ExitMenu => {
                ui.save_exit_menu.navigate_up();
            }
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } if *game_state == GameState::ExitMenu => {
                ui.save_exit_menu.navigate_down();
            }
            Event::KeyDown {
                keycode: Some(Keycode::Return | Keycode::Space),
                ..
            } if *game_state == GameState::ExitMenu => {
                match ui.save_exit_menu.selected_option() {
                    SaveExitOption::SaveAndExit => {
                        if let Err(e) = save_game(save_manager, &world.player, &world.slimes, &world.world_grid, &world.entities, &world.player_inventory, &world.dropped_items) {
                            eprintln!("Failed to save: {}", e);
                        }
                        return Ok(true);
                    }
                    SaveExitOption::Cancel => {
                        *game_state = GameState::Playing;
                    }
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::F5),
                ..
            } if *game_state == GameState::Playing => {
                if let Err(e) = save_game(save_manager, &world.player, &world.slimes, &world.world_grid, &world.entities, &world.player_inventory, &world.dropped_items) {
                    eprintln!("Failed to save: {}", e);
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::F9),
                ..
            } if *game_state == GameState::Playing => {
                match load_game(save_manager, &systems.player_config, &systems.slime_config, character_texture, slime_texture, entity_texture, item_textures) {
                    Ok((loaded_player, loaded_slimes, loaded_world, loaded_entities, loaded_inventory, loaded_items)) => {
                        world.player = loaded_player;
                        world.slimes = loaded_slimes;
                        world.world_grid = loaded_world;
                        world.render_grid = RenderGrid::new(&world.world_grid);
                        world.entities = loaded_entities;
                        world.player_inventory = loaded_inventory;
                        world.dropped_items = loaded_items;
                        world.attack_effects.clear();
                        world.active_attack = None;
                        println!("✓ Game loaded successfully!");
                    }
                    Err(e) => {
                        eprintln!("Failed to load: {}", e);
                    }
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::Return | Keycode::Space),
                ..
            } if matches!(ui.debug_menu_state, DebugMenuState::Open { .. }) => {
                if let DebugMenuState::Open { selected_index } = ui.debug_menu_state {
                    let items = DebugMenuItem::all();
                    match items[selected_index] {
                        DebugMenuItem::ClearInventory => {
                            world.player_inventory.inventory.clear();
                            println!("Player inventory cleared!");
                        }
                        _ => { /* Other debug menu items don't have an action on Enter/Space */ }
                    }
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::F3),
                ..
            } if *game_state == GameState::Playing => {
                ui.debug_menu_state = match ui.debug_menu_state {
                    DebugMenuState::Closed => {
                        println!("Debug menu: OPEN");
                        DebugMenuState::Open { selected_index: 0 }
                    }
                    DebugMenuState::Open { .. } => {
                        println!("Debug menu: CLOSED");
                        DebugMenuState::Closed
                    }
                };
            }
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } if matches!(ui.debug_menu_state, DebugMenuState::Open { .. }) => {
                if let DebugMenuState::Open { selected_index } = ui.debug_menu_state {
                    let items = DebugMenuItem::all();
                    let new_index = if selected_index == 0 {
                        items.len() - 1
                    } else {
                        selected_index - 1
                    };
                    ui.debug_menu_state = DebugMenuState::Open { selected_index: new_index };
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } if matches!(ui.debug_menu_state, DebugMenuState::Open { .. }) => {
                if let DebugMenuState::Open { selected_index } = ui.debug_menu_state {
                    let items = DebugMenuItem::all();
                    let new_index = (selected_index + 1) % items.len();
                    ui.debug_menu_state = DebugMenuState::Open { selected_index: new_index };
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                keymod,
                ..
            } if matches!(ui.debug_menu_state, DebugMenuState::Open { .. }) => {
                if let DebugMenuState::Open { selected_index } = ui.debug_menu_state {
                    let items = DebugMenuItem::all();
                    let item = items[selected_index];
                    let shift_held = keymod.intersects(sdl2::keyboard::Mod::LSHIFTMOD | sdl2::keyboard::Mod::RSHIFTMOD);
                    let delta = if shift_held { -10.0 } else { -1.0 };

                    match item {
                        DebugMenuItem::PlayerMaxHealth => {
                            let new_val = (world.player.stats.max_health + delta).max(1.0);
                            world.player.stats.max_health = new_val;
                            world.player.stats.health.set_max(new_val);
                        }
                        DebugMenuItem::PlayerAttackDamage => {
                            world.player.stats.attack_damage = (world.player.stats.attack_damage + delta).max(0.0);
                        }
                        DebugMenuItem::PlayerAttackSpeed => {
                            world.player.stats.attack_speed = (world.player.stats.attack_speed + delta).max(0.1);
                        }
                        DebugMenuItem::SlimeHealth => {
                            systems.debug_config.slime_base_health = (systems.debug_config.slime_base_health as f32 + delta).max(1.0) as i32;
                        }
                        DebugMenuItem::SlimeContactDamage => {
                            systems.debug_config.slime_contact_damage = (systems.debug_config.slime_contact_damage + delta).max(0.0);
                        }
                        DebugMenuItem::ClearInventory => {
                            // This item doesn't change value with left/right, it's an action.
                        }
                    }
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                keymod,
                ..
            } if matches!(ui.debug_menu_state, DebugMenuState::Open { .. }) => {
                if let DebugMenuState::Open { selected_index } = ui.debug_menu_state {
                    let items = DebugMenuItem::all();
                    let item = items[selected_index];
                    let shift_held = keymod.intersects(sdl2::keyboard::Mod::LSHIFTMOD | sdl2::keyboard::Mod::RSHIFTMOD);
                    let delta = if shift_held { 10.0 } else { 1.0 };

                    match item {
                        DebugMenuItem::PlayerMaxHealth => {
                            let new_val = world.player.stats.max_health + delta;
                            world.player.stats.max_health = new_val;
                            world.player.stats.health.set_max(new_val);
                        }
                        DebugMenuItem::PlayerAttackDamage => {
                            world.player.stats.attack_damage += delta;
                        }
                        DebugMenuItem::PlayerAttackSpeed => {
                            world.player.stats.attack_speed += delta;
                        }
                        DebugMenuItem::SlimeHealth => {
                            systems.debug_config.slime_base_health = (systems.debug_config.slime_base_health as f32 + delta) as i32;
                        }
                        DebugMenuItem::SlimeContactDamage => {
                            systems.debug_config.slime_contact_damage += delta;
                        }
                        DebugMenuItem::ClearInventory => {
                            // This item doesn't change value with left/right, it's an action.
                        }
                    }
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::M),
                ..
            } if !is_ui_active => {
                if let Some(attack_event) = world.player.start_attack() {
                    world.active_attack = Some(attack_event.clone());

                    // Attack effect positioning
                    // Player position is now anchor-based (bottom-center)
                    // We want the effect centered on the player's body, then offset by direction

                    // Calculate player's visual center from anchor
                    let player_center_y = world.player.y - (world.player.height * SPRITE_SCALE) as i32 / 2;

                    // Directional offset from player center
                    // ADJUST THIS VALUE to change attack effect distance
                    let offset = 20;
                    let (offset_x, offset_y) = match world.player.direction {
                        crate::animation::Direction::North => (0, -offset),
                        crate::animation::Direction::NorthEast => (offset, -offset),
                        crate::animation::Direction::East => (offset, 0),
                        crate::animation::Direction::SouthEast => (offset, offset),
                        crate::animation::Direction::South => (0, offset),
                        crate::animation::Direction::SouthWest => (-offset, offset),
                        crate::animation::Direction::West => (-offset, 0),
                        crate::animation::Direction::NorthWest => (-offset, -offset),
                    };

                    // Calculate effect center position
                    let effect_center_x = world.player.x + offset_x;
                    let effect_center_y = player_center_y + offset_y;

                    // Convert to top-left (AttackEffect uses top-left positioning)
                    // Effect is 32x32 at 2x scale = 64x64
                    let effect_size = 32 * SPRITE_SCALE as i32;
                    let effect_x = effect_center_x - effect_size / 2;
                    let effect_y = effect_center_y - effect_size / 2;

                    match systems.punch_config.create_controller(punch_texture, &["punch"]) {
                        Ok(punch_animation_controller) => {
                            world.attack_effects.push(AttackEffect::new(
                                effect_x,
                                effect_y,
                                32,
                                32,
                                world.player.direction,
                                punch_animation_controller,
                            ));
                        }
                        Err(e) => {
                            eprintln!("ERROR: Failed to create punch controller: {}", e);
                        }
                    }
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::B),
                ..
            } if !is_ui_active => {
                ui.show_collision_boxes = !ui.show_collision_boxes;
                println!("Collision boxes: {}", if ui.show_collision_boxes { "ON" } else { "OFF" });
            }
            Event::KeyDown {
                keycode: Some(Keycode::G),
                ..
            } if !is_ui_active => {
                ui.show_tile_grid = !ui.show_tile_grid;
                println!("Tile grid debug: {}", if ui.show_tile_grid { "ON" } else { "OFF" });
            }
            Event::KeyDown {
                keycode: Some(Keycode::I),
                ..
            } => {
                ui.inventory_ui.toggle();
            }
            // Number keys (1-9) to select hotbar slots
            Event::KeyDown {
                keycode: Some(Keycode::Num1),
                ..
            } if !is_ui_active => {
                world.player_inventory.set_hotbar_slot(0);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num2),
                ..
            } if !is_ui_active => {
                world.player_inventory.set_hotbar_slot(1);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num3),
                ..
            } if !is_ui_active => {
                world.player_inventory.set_hotbar_slot(2);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num4),
                ..
            } if !is_ui_active => {
                world.player_inventory.set_hotbar_slot(3);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num5),
                ..
            } if !is_ui_active => {
                world.player_inventory.set_hotbar_slot(4);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num6),
                ..
            } if !is_ui_active => {
                world.player_inventory.set_hotbar_slot(5);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num7),
                ..
            } if !is_ui_active => {
                world.player_inventory.set_hotbar_slot(6);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num8),
                ..
            } if !is_ui_active => {
                world.player_inventory.set_hotbar_slot(7);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num9),
                ..
            } if !is_ui_active => {
                world.player_inventory.set_hotbar_slot(8);
            }
            Event::KeyDown {
                keycode: Some(Keycode::P),
                ..
            } if ui.inventory_ui.is_open => {
                // Give a stack of 16 slime balls to the player
                match world.player_inventory.quick_add("slime_ball", 16, item_registry) {
                    Ok(0) => println!("Added 16 slime balls to inventory."),
                    Ok(overflow) => println!("Inventory full. {} slime balls could not be added.", overflow),
                    Err(e) => eprintln!("Error adding slime balls: {}", e),
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::H),
                ..
            } if ui.inventory_ui.is_open => {
                // Give a hoe to the player (debug command)
                match world.player_inventory.quick_add("hoe", 1, item_registry) {
                    Ok(0) => println!("Added hoe to inventory."),
                    Ok(_overflow) => println!("Inventory full. Hoe could not be added."),
                    Err(e) => eprintln!("Error adding hoe: {}", e),
                }
            }
            Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                // Handle inventory/hotbar clicks (hotbar works even when inventory is closed)
                if *game_state == GameState::Playing {
                    let (screen_width, screen_height) = canvas.logical_size();
                    // TODO: Add shift-click support for inventory. Currently can't check keyboard_state
                    // during event loop due to borrowing constraints. Need to refactor event handling.
                    let shift_held = false;
                    ui.inventory_ui.handle_mouse_click(x, y, screen_width, screen_height, &mut world.player_inventory, shift_held, mouse_btn)?;
                }

                if !is_ui_active {
                    if mouse_btn == sdl2::mouse::MouseButton::Right {
                        let slime_animation_controller = systems.slime_config.create_controller(
                            slime_texture,
                            &["slime_idle", "jump", "slime_damage", "slime_death"],
                        )?;

                        // Spawn slime so that click position = collision box center
                        // This makes spawning intuitive and easier for procedural generation
                        //
                        // Slime uses anchor positioning internally (bottom-center of sprite)
                        // but we want the click to target the collision box center (visible body)
                        //
                        // To calculate anchor from desired collision center:
                        // anchor = click - collision_offset - (collision_size / 2)
                        let temp_slime = Slime::new(0, 0, AnimationController::new());
                        let anchor_x = x - (temp_slime.hitbox_offset_x * SPRITE_SCALE as i32) - (temp_slime.hitbox_width * SPRITE_SCALE / 2) as i32;
                        let anchor_y = y - (temp_slime.hitbox_offset_y * SPRITE_SCALE as i32) - (temp_slime.hitbox_height * SPRITE_SCALE / 2) as i32;

                        let mut new_slime = Slime::new(anchor_x, anchor_y, slime_animation_controller);
                        new_slime.health = systems.debug_config.slime_base_health;
                        world.slimes.push(new_slime);
                    }

                    // Check if player has a hoe selected in hotbar
                    if let Some(selected_item) = world.player_inventory.get_selected_hotbar() {
                        if let Some(item_def) = item_registry.get(&selected_item.item_id) {
                            if let ItemProperties::Tool { tool_type: ToolType::Hoe, .. } = item_def.properties {
                                // Player has a hoe selected, allow tilling
                                ui.is_tilling = true;
                                let tile_x = x / 32;
                                let tile_y = y / 32;

                                // Only allow grass -> dirt conversion
                                if world.world_grid.get_tile(tile_x, tile_y) == Some(TileId::Grass) {
                                    if world.world_grid.set_tile(tile_x, tile_y, TileId::Dirt) {
                                        world.render_grid.update_tile_and_neighbors(&world.world_grid, tile_x, tile_y);
                                        ui.last_tilled_tile = Some((tile_x, tile_y));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Event::MouseButtonUp { mouse_btn: sdl2::mouse::MouseButton::Left, x, y, .. } => {
                ui.is_tilling = false;
                ui.last_tilled_tile = None;

                // If an item is held and mouse is released outside all inventory UI (hotbar + window), drop it
                if *game_state == GameState::Playing && ui.inventory_ui.held_item.is_some() {
                    let (screen_width, screen_height) = canvas.logical_size();
                    if !ui.inventory_ui.is_mouse_over_any_inventory(x, y, screen_width, screen_height) {
                        if let Some(item_stack) = ui.inventory_ui.held_item.take() {
                            // Spawn dropped item at player's position
                            let mut item_animation_controller = animation::AnimationController::new();
                            let item_frames = vec![
                                sprite::Frame::new(0, 0, 32, 32, 300),
                            ];
                            let item_texture = item_textures.get(&item_stack.item_id).ok_or(format!("Missing texture for item {}", item_stack.item_id))?;
                            let item_sprite_sheet = SpriteSheet::new(item_texture, item_frames);
                            item_animation_controller.add_animation("item_idle".to_string(), item_sprite_sheet);
                            item_animation_controller.set_state("item_idle".to_string());

                            let dropped_item = DroppedItem::new(
                                world.player.x,
                                world.player.y,
                                item_stack.item_id.clone(),
                                item_stack.quantity,
                                item_animation_controller,
                            );
                            world.dropped_items.push(dropped_item);
                            println!("Dropped {} x{} at ({}, {})", item_stack.item_id, item_stack.quantity, world.player.x, world.player.y);
                        }
                    }
                }
            }
            Event::MouseMotion { x, y, .. } => {
                ui.mouse_x = x;
                ui.mouse_y = y;
                if ui.is_tilling && !is_ui_active {
                    let tile_x = x / 32;
                    let tile_y = y / 32;

                    if ui.last_tilled_tile != Some((tile_x, tile_y)) {
                        // Only allow grass -> dirt conversion
                        if world.world_grid.get_tile(tile_x, tile_y) == Some(TileId::Grass) {
                            if world.world_grid.set_tile(tile_x, tile_y, TileId::Dirt) {
                                world.render_grid.update_tile_and_neighbors(&world.world_grid, tile_x, tile_y);
                                ui.last_tilled_tile = Some((tile_x, tile_y));
                            }
                        }
                    }
                }
            }

            _ => {} // Ignore other events
        }
    }

    Ok(false) // Don't quit
}

/// Update game world state (entities, collisions, physics, loot)
/// This is called once per frame when game_state == Playing
#[allow(clippy::too_many_arguments)]
fn update_world<'a>(
    world: &mut GameWorld<'a>,
    systems: &mut Systems,
    item_textures: &'a HashMap<String, sdl2::render::Texture<'a>>,
    item_registry: &ItemRegistry,
    keyboard_state: &sdl2::keyboard::KeyboardState,
) -> Result<(), String> {
    // Update player movement
    world.player.update(keyboard_state);

    // Handle active attack hitting enemies
    if let Some(ref attack) = world.active_attack {
        let attack_hitbox = attack.get_hitbox();

        // Check attack vs slimes
        for slime in world.slimes.iter_mut() {
            let slime_bounds = slime.get_bounds();
            if collision::aabb_intersect(&attack_hitbox, &slime_bounds) {
                slime.take_damage(attack.damage as i32);
            }
        }

        // Check attack vs entities (pyramids)
        for entity in world.entities.iter_mut() {
            if let Some(state_before_hit) = entity.check_hit(&attack_hitbox) {
                // Drop stone only if entity was not in Awake state
                if state_before_hit != the_entity::EntityState::Awake {
                    let drop_x = entity.x + (entity.width as i32) / 2;
                    let drop_y = entity.y + (entity.height as i32) / 2;

                    let mut item_animation_controller = animation::AnimationController::new();
                    let item_frames = vec![sprite::Frame::new(0, 0, 32, 32, 300)];

                    if let Some(item_texture) = item_textures.get("stone") {
                        let item_sprite_sheet = sprite::SpriteSheet::new(item_texture, item_frames);
                        item_animation_controller.add_animation("item_idle".to_string(), item_sprite_sheet);
                        item_animation_controller.set_state("item_idle".to_string());

                        let dropped_item = dropped_item::DroppedItem::new(
                            drop_x,
                            drop_y,
                            "stone".to_string(),
                            1,
                            item_animation_controller,
                        );
                        world.dropped_items.push(dropped_item);
                    }
                }
            }
        }

        world.active_attack = None;
    }

    // Update slimes
    for slime in world.slimes.iter_mut() {
        slime.update();
    }

    // Update entities and floating texts
    let delta_time = 1.0 / 60.0;
    for entity in world.entities.iter_mut() {
        entity.update(delta_time);
    }

    for text in world.floating_texts.iter_mut() {
        text.lifetime += delta_time;
        text.y -= 20.0 * delta_time;
    }
    world.floating_texts.retain(|text| text.lifetime < text.max_lifetime);

    // Apply entity buffs to player
    world.player.active_modifiers.clear();
    systems.has_regen = false;
    for entity in world.entities.iter() {
        if entity.state == EntityState::Awake {
            match entity.entity_type {
                EntityType::Attack => {
                    world.player.active_modifiers.push(ModifierEffect {
                        stat_type: StatType::AttackDamage,
                        modifier: StatModifier::Flat(1.0),
                        duration: None,
                        source: "Pyramid of Attack".to_string(),
                    });
                }
                EntityType::Defense => {
                    world.player.active_modifiers.push(ModifierEffect {
                        stat_type: StatType::Defense,
                        modifier: StatModifier::Flat(1.0),
                        duration: None,
                        source: "Pyramid of Defense".to_string(),
                    });
                }
                EntityType::Speed => {
                    world.player.active_modifiers.push(ModifierEffect {
                        stat_type: StatType::MovementSpeed,
                        modifier: StatModifier::Flat(1.0),
                        duration: None,
                        source: "Pyramid of Speed".to_string(),
                    });
                }
                EntityType::Regeneration => {
                    systems.has_regen = true;
                }
            }
        }
    }

    // Handle regeneration
    if systems.has_regen && systems.regen_timer.elapsed().as_secs_f32() >= systems.regen_interval {
        if world.player.stats.health.current() < world.player.stats.max_health {
            world.player.stats.health.heal(2.0);

            world.floating_texts.push(FloatingTextInstance {
                x: world.player.x as f32,
                y: (world.player.y - (world.player.height * SPRITE_SCALE) as i32) as f32,
                text: "+2".to_string(),
                color: Color::RGB(0, 255, 0),
                lifetime: 0.0,
                max_lifetime: 1.5,
            });

            for entity in world.entities.iter() {
                if entity.state == EntityState::Awake && entity.entity_type == EntityType::Regeneration {
                    world.floating_texts.push(FloatingTextInstance {
                        x: entity.x as f32 + 16.0,
                        y: entity.y as f32,
                        text: "+2".to_string(),
                        color: Color::RGB(0, 255, 0),
                        lifetime: 0.0,
                        max_lifetime: 1.5,
                    });
                    break;
                }
            }
        }
        systems.regen_timer = Instant::now();
    }

    // Update attack effects
    for effect in world.attack_effects.iter_mut() {
        effect.update();
    }
    world.attack_effects.retain(|effect| !effect.is_finished());

    // Handle player-slime collisions (push physics + contact damage)
    let colliding_slime_indices = check_collisions_with_collection(&world.player, &world.slimes);

    for slime_index in colliding_slime_indices {
        let player_bounds = world.player.get_bounds();
        let slime_bounds = world.slimes[slime_index].get_bounds();

        let (overlap_x, overlap_y) = calculate_overlap(&player_bounds, &slime_bounds);

        // Push-apart physics (30% player, 70% slime)
        if overlap_x.abs() < overlap_y.abs() {
            world.player.apply_push(-overlap_x * 3 / 10, 0);
            world.slimes[slime_index].apply_push(overlap_x * 7 / 10, 0);
        } else {
            world.player.apply_push(0, -overlap_y * 3 / 10);
            world.slimes[slime_index].apply_push(0, overlap_y * 7 / 10);
        }

        // Contact damage
        if !world.player.is_attacking && !world.slimes[slime_index].is_invulnerable() {
            let damage = DamageEvent::physical(systems.debug_config.slime_contact_damage, DamageSource::Enemy);
            let damage_result = world.player.take_damage(damage);
            if damage_result.is_fatal {
                // Drop all items from player inventory
                for item_stack_option in world.player_inventory.inventory.slots.iter_mut() {
                    if let Some(item_stack) = item_stack_option.take() {
                        let mut item_animation_controller = animation::AnimationController::new();
                        let item_frames = vec![sprite::Frame::new(0, 0, 32, 32, 300)];
                        let item_texture = item_textures.get(&item_stack.item_id).ok_or(format!("Missing texture for item {}", item_stack.item_id))?;
                        let item_sprite_sheet = SpriteSheet::new(item_texture, item_frames);
                        item_animation_controller.add_animation("item_idle".to_string(), item_sprite_sheet);
                        item_animation_controller.set_state("item_idle".to_string());

                        let dropped_item = DroppedItem::new(
                            world.player.x,
                            world.player.y,
                            item_stack.item_id.clone(),
                            item_stack.quantity,
                            item_animation_controller,
                        );
                        world.dropped_items.push(dropped_item);
                    }
                }
                println!("Player died and dropped all items.");
            }
        }
    }

    // Handle slime loot drops
    for slime in world.slimes.iter_mut() {
        if slime.is_dying() && !slime.has_dropped_loot {
            slime.has_dropped_loot = true;
            let mut item_animation_controller = animation::AnimationController::new();
            let item_frames = vec![sprite::Frame::new(0, 0, 32, 32, 300)];
            let item_texture = item_textures.get("slime_ball").ok_or("Missing slime_ball texture in item_textures map")?;
            let item_sprite_sheet = SpriteSheet::new(item_texture, item_frames);
            item_animation_controller.add_animation("item_idle".to_string(), item_sprite_sheet);
            item_animation_controller.set_state("item_idle".to_string());

            let drop_x = slime.x + (slime.hitbox_offset_x * SPRITE_SCALE as i32) + (slime.hitbox_width * SPRITE_SCALE / 2) as i32;
            let drop_y = slime.y + (slime.hitbox_offset_y * SPRITE_SCALE as i32) + (slime.hitbox_height * SPRITE_SCALE / 2) as i32;

            let dropped_item = DroppedItem::new(
                drop_x,
                drop_y,
                "slime_ball".to_string(),
                1,
                item_animation_controller,
            );
            world.dropped_items.push(dropped_item);
        }
    }

    // Remove dead slimes
    world.slimes.retain(|slime| slime.is_alive);

    // Handle item pickup
    let player_bounds = world.player.get_bounds();
    world.dropped_items.retain(|item| {
        if !item.can_pickup {
            return true; // Keep items in cooldown
        }

        if player_bounds.has_intersection(item.get_bounds()) {
            match world.player_inventory.quick_add(&item.item_id, item.quantity, item_registry) {
                Ok(overflow) => {
                    if overflow == 0 {
                        println!("✓ Picked up {} x{}", item.item_id, item.quantity);
                        false // Remove from world
                    } else {
                        println!("⚠ Inventory full! {} items couldn't fit", overflow);
                        true // Keep in world
                    }
                }
                Err(e) => {
                    eprintln!("Failed to pickup item: {}", e);
                    true // Keep in world on error
                }
            }
        } else {
            true // Keep if not colliding
        }
    });

    // Update dropped items (for despawn timer)
    world.dropped_items.retain_mut(|item| !item.update()); // Remove if despawned

    // Handle player-static collisions (walls, entities)
    let mut all_static_collidables: Vec<&dyn StaticCollidable> = Vec::new();
    for obj in &systems.static_objects {
        all_static_collidables.push(obj);
    }
    for entity in world.entities.iter() {
        all_static_collidables.push(entity);
    }

    let static_collisions = check_static_collisions(&world.player, &all_static_collidables);

    for obj_index in static_collisions {
        let player_bounds = world.player.get_bounds();
        let obj_bounds = all_static_collidables[obj_index].get_bounds();

        let (overlap_x, overlap_y) = calculate_overlap(&player_bounds, &obj_bounds);

        // Push player out completely (100% push on player, 0% on static object)
        if overlap_x.abs() < overlap_y.abs() {
            world.player.apply_push(-overlap_x, 0);
        } else {
            world.player.apply_push(0, -overlap_y);
        }
    }

    Ok(())
}


/// Generic texture loading helper
///
/// Loads a texture from the given path with consistent error handling
fn load_texture<'a>(
    texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    path: &str,
) -> Result<sdl2::render::Texture<'a>, String> {
    texture_creator
        .load_texture(path)
        .map_err(|e| format!("Failed to load {}: {}", path, e))
}

// REMOVED: Old repetitive setup functions replaced with AnimationConfig::create_controller()!
//
// Game Dev Pattern: Don't Repeat Yourself (DRY)
// The old code had 50+ lines of boilerplate that's now replaced by single-line calls:
//   config.create_controller(texture, &["idle", "running", "attack"])?
//
// Benefits:
// - Less code = fewer bugs
// - Easier to add new entities (no new function needed)
// - Configuration-driven (JSON defines what exists)
// - Factory pattern encapsulates complexity

/// Calculate the best window scale based on monitor size
fn calculate_window_scale(video_subsystem: &sdl2::VideoSubsystem) -> u32 {
    match video_subsystem.desktop_display_mode(0) {
        Ok(display_mode) => {
            // Leave 10% margin for taskbars/decorations
            let usable_w = (display_mode.w as f32 * 0.9) as i32;
            let usable_h = (display_mode.h as f32 * 0.9) as i32;

            let max_scale_w = usable_w / GAME_WIDTH as i32;
            let max_scale_h = usable_h / GAME_HEIGHT as i32;

            // Use smaller scale to ensure both dimensions fit
            let scale = max_scale_w.min(max_scale_h);

            // Clamp to reasonable range (2x minimum, 6x maximum)
            scale.clamp(2, 6) as u32
        }
        Err(_) => {
            // Fallback to 2x if monitor detection fails
            println!("Warning: Could not detect monitor size, using 2x scale");
            2
        }
    }
}

/// Load game state from save file
fn load_game<'a>(
    save_manager: &SaveManager,
    player_config: &AnimationConfig,
    slime_config: &AnimationConfig,
    character_texture: &'a sdl2::render::Texture<'a>,
    slime_texture: &'a sdl2::render::Texture<'a>,
    entity_texture: &'a sdl2::render::Texture<'a>,
    item_textures: &'a HashMap<String, sdl2::render::Texture<'a>>,
) -> Result<(Player<'a>, Vec<Slime<'a>>, WorldGrid, Vec<TheEntity<'a>>, PlayerInventory, Vec<DroppedItem<'a>>), String> {
    // Load save file from slot 1
    let save_file = save_manager.load_game(1)
        .map_err(|e| format!("Failed to load save: {}", e))?;

    println!("Loading game...");
    println!("  - Save version: {}", save_file.version);
    println!("  - Saved: {:?}", save_file.timestamp);

    // Load world
    let world_grid = WorldGrid::from_save_data(
        save_file.world_state.width,
        save_file.world_state.height,
        save_file.world_state.tiles,
    ).ok_or_else(|| "Failed to load world grid".to_string())?;

    println!("  - Loaded world: {}x{} tiles", world_grid.width, world_grid.height);

    // Load entities
    let mut player: Option<Player> = None;
    let mut slimes: Vec<Slime> = Vec::new();
    let mut loaded_entities: Vec<TheEntity> = Vec::new();
    let mut player_inventory = PlayerInventory::new();
    let mut dropped_items = Vec::new();

    for entity_data in save_file.entities {
        match entity_data.entity_type.as_str() {
            "player" => {
                let save_data = SaveData {
                    data_type: "player".to_string(),
                    json_data: entity_data.data,
                };

                let mut loaded_player = Player::from_save_data(&save_data)
                    .map_err(|e| format!("Failed to load player: {}", e))?;

                let animation_controller = player_config.create_controller(
                    character_texture,
                    &["idle", "running", "attack", "damage", "death"],
                ).map_err(|e| format!("Failed to create player animations: {}", e))?;

                loaded_player.set_animation_controller(animation_controller);
                player = Some(loaded_player);
                println!("  - Loaded player at ({}, {})", entity_data.position.0, entity_data.position.1);
            }
            "slime" => {
                let save_data = SaveData {
                    data_type: "slime".to_string(),
                    json_data: entity_data.data,
                };

                let mut loaded_slime = Slime::from_save_data(&save_data)
                    .map_err(|e| format!("Failed to load slime: {}", e))?;

                let slime_animation_controller = slime_config.create_controller(
                    slime_texture,
                    &["slime_idle", "jump", "slime_damage", "slime_death"],
                ).map_err(|e| format!("Failed to create slime animations: {}", e))?;

                loaded_slime.set_animation_controller(slime_animation_controller);
                slimes.push(loaded_slime);
            }
            "the_entity" => {
                #[derive(Deserialize)]
                struct EntitySaveData {
                    id: usize,
                    x: i32,
                    y: i32,
                    state: EntityState,
                    awakening_frame: usize,
                    inactivity_timer: f32,
                    entity_type: EntityType,
                }

                let saved_entity: EntitySaveData = serde_json::from_str(&entity_data.data)
                    .map_err(|e| format!("Failed to deserialize entity: {}", e))?;

                let mut frames = Vec::new();
                for i in 0..13 {
                    frames.push(sprite::Frame::new(i * 32, 0, 32, 32, 100));
                }
                let sprite_sheet = sprite::SpriteSheet::new(entity_texture, frames);

                let mut loaded_entity = TheEntity::new(saved_entity.id, saved_entity.x, saved_entity.y, saved_entity.entity_type, sprite_sheet);
                loaded_entity.state = saved_entity.state;
                loaded_entity.awakening_frame = saved_entity.awakening_frame;
                loaded_entity.inactivity_timer = saved_entity.inactivity_timer;

                loaded_entity.update_sprite_frame();

                loaded_entities.push(loaded_entity);
            }
            "player_inventory" => {
                player_inventory = serde_json::from_str(&entity_data.data).map_err(|e| format!("Failed to load player inventory: {}", e))?;
            }
            "dropped_item" => {
                let save_data = SaveData {
                    data_type: "dropped_item".to_string(),
                    json_data: entity_data.data,
                };
                let mut item = DroppedItem::from_save_data(&save_data).map_err(|e| e.to_string())?;
                let mut item_animation_controller = animation::AnimationController::new();
                let item_frames = vec![
                    sprite::Frame::new(0, 0, 32, 32, 300),
                ];
                let item_texture = item_textures.get(&item.item_id).ok_or(format!("Missing texture for item {}", item.item_id))?;
                let item_sprite_sheet = SpriteSheet::new(item_texture, item_frames);
                item_animation_controller.add_animation("item_idle".to_string(), item_sprite_sheet);
                item_animation_controller.set_state("item_idle".to_string());
                item.set_animation_controller(item_animation_controller);
                dropped_items.push(item);
            }
            unknown => {
                eprintln!("Warning: Unknown entity type '{}', skipping", unknown);
            }
        }
    }

    let player = player.ok_or_else(|| "No player found in save file".to_string())?;
    println!("  - Loaded {} slimes", slimes.len());
    println!("  - Loaded {} entities", loaded_entities.len());
    println!("✓ Game loaded successfully!");

    Ok((player, slimes, world_grid, loaded_entities, player_inventory, dropped_items))
}

/// Save the current game state
fn save_game(
    save_manager: &mut SaveManager,
    player: &Player,
    slimes: &[Slime],
    world_grid: &WorldGrid,
    the_entities: &[TheEntity],
    player_inventory: &PlayerInventory,
    dropped_items: &[DroppedItem],
) -> Result<(), String> {
    let mut entities_vec = Vec::new();

    let player_save_data = player.to_save_data()
        .map_err(|e| format!("Failed to save player: {}", e))?;

    entities_vec.push(EntitySaveData {
        entity_id: 0,
        entity_type: "player".to_string(),
        position: player.position(),
        data: player_save_data.json_data,
    });

    let mut next_id = 1;
    for (i, slime) in slimes.iter().enumerate() {
        let slime_save_data = slime.to_save_data()
            .map_err(|e| format!("Failed to save slime {}: {}", i, e))?;

        entities_vec.push(EntitySaveData {
            entity_id: (next_id + i) as u64,
            entity_type: "slime".to_string(),
            position: (slime.x, slime.y),
            data: slime_save_data.json_data,
        });
    }
    next_id += slimes.len();

    for entity in the_entities.iter() {
        let entity_save_data = entity.to_save_data()
            .map_err(|e| format!("Failed to save entity {}: {}", entity.id, e))?;

        entities_vec.push(EntitySaveData {
            entity_id: (next_id + entity.id) as u64,
            entity_type: "the_entity".to_string(),
            position: (entity.x, entity.y),
            data: entity_save_data.json_data,
        });
    }
    next_id += the_entities.len();

    let inventory_data = serde_json::to_string(player_inventory).map_err(|e| format!("Failed to serialize player inventory: {}", e))?;
    entities_vec.push(EntitySaveData {
        entity_id: u64::MAX - 1, // Special ID for inventory
        entity_type: "player_inventory".to_string(),
        position: (0, 0),
        data: inventory_data,
    });

    for (i, item) in dropped_items.iter().enumerate() {
        let item_save_data = item.to_save_data().map_err(|e| format!("Failed to save dropped item: {}", e))?;
        entities_vec.push(EntitySaveData {
            entity_id: (next_id + i) as u64,
            entity_type: "dropped_item".to_string(),
            position: (item.x, item.y),
            data: item_save_data.json_data,
        });
    }

    let world_state = WorldSaveData {
        width: world_grid.width,
        height: world_grid.height,
        tiles: world_grid.to_save_data(),
    };

    let entity_count = entities_vec.len();
    let slime_count = slimes.len();
    let entity_pyramid_count = the_entities.len();
    let world_width = world_state.width;
    let world_height = world_state.height;

    let save_file = SaveFile {
        version: CURRENT_SAVE_VERSION,
        timestamp: SystemTime::now(),
        metadata: SaveMetadata {
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            player_name: None,
            playtime_seconds: 0, // TODO: track playtime
            save_type: SaveType::Manual,
            save_slot: save_manager.get_save_slot(),
        },
        world_state,
        entities: entities_vec,
    };

    save_manager.save_game(&save_file)
        .map_err(|e| format!("Save failed: {}", e))?;

    println!("✓ Game saved successfully!");
    println!("  - Saved {} entities ({} slimes, {} pyramids)", entity_count, slime_count, entity_pyramid_count);
    println!("  - Saved world: {}x{} tiles", world_width, world_height);
    Ok(())
}

/// Render the debug stats menu overlay
fn render_debug_menu(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    player: &Player,
    debug_config: &DebugConfig,
    selected_index: usize,
) -> Result<(), String> {
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 200));
    canvas.fill_rect(None)?;
    canvas.set_blend_mode(sdl2::render::BlendMode::None);

    let menu_width = 380;
    let menu_height = 260;
    let menu_x = (GAME_WIDTH - menu_width) / 2;
    let menu_y = (GAME_HEIGHT - menu_height) / 2;

    canvas.set_draw_color(sdl2::pixels::Color::RGB(20, 20, 30));
    canvas.fill_rect(sdl2::rect::Rect::new(
        menu_x as i32,
        menu_y as i32,
        menu_width,
        menu_height,
    ))?;

    canvas.set_draw_color(sdl2::pixels::Color::RGB(80, 120, 180));
    canvas.draw_rect(sdl2::rect::Rect::new(
        menu_x as i32,
        menu_y as i32,
        menu_width,
        menu_height,
    ))?;
    canvas.draw_rect(sdl2::rect::Rect::new(
        (menu_x + 2) as i32,
        (menu_y + 2) as i32,
        menu_width - 4,
        menu_height - 4,
    ))?;

    draw_simple_text(
        canvas,
        "DEBUG STATS",
        (menu_x + 110) as i32,
        (menu_y + 15) as i32,
        sdl2::pixels::Color::RGB(180, 220, 255),
        2,
    )?;

    draw_simple_text(
        canvas,
        "F3 TO CLOSE",
        (menu_x + 130) as i32,
        (menu_y + 35) as i32,
        sdl2::pixels::Color::RGB(120, 140, 160),
        1,
    )?;

    let items = DebugMenuItem::all();
    let item_y_start = menu_y + 60;
    let item_height = 35;

    for (i, item) in items.iter().enumerate() {
        let item_y = item_y_start + (i as u32 * item_height);
        let is_selected = i == selected_index;

        if is_selected {
            canvas.set_draw_color(sdl2::pixels::Color::RGB(40, 60, 100));
            canvas.fill_rect(sdl2::rect::Rect::new(
                (menu_x + 10) as i32,
                item_y as i32 - 2,
                menu_width - 20,
                30,
            ))?;
        }

        let text_color = if is_selected {
            sdl2::pixels::Color::RGB(255, 255, 255)
        } else {
            sdl2::pixels::Color::RGB(180, 180, 190)
        };

        draw_simple_text(
            canvas,
            item.name(),
            (menu_x + 20) as i32,
            item_y as i32,
            text_color,
            2,
        )?;

        let value_text = match item {
            DebugMenuItem::PlayerMaxHealth => format!("{}", player.stats.max_health as i32),
            DebugMenuItem::PlayerAttackDamage => format!("{}", player.stats.attack_damage as i32),
            DebugMenuItem::PlayerAttackSpeed => format!("{:.1}", player.stats.attack_speed),
            DebugMenuItem::SlimeHealth => format!("{}", debug_config.slime_base_health),
            DebugMenuItem::SlimeContactDamage => format!("{:.1}", debug_config.slime_contact_damage),
            DebugMenuItem::ClearInventory => "".to_string(), // No value to display
        };

        draw_simple_text(
            canvas,
            &value_text,
            (menu_x + 280) as i32,
            item_y as i32,
            sdl2::pixels::Color::RGB(100, 255, 100),
            2,
        )?;

        if is_selected {
            draw_simple_text(
                canvas,
                "<",
                (menu_x + 260) as i32,
                item_y as i32,
                sdl2::pixels::Color::RGB(255, 200, 100),
                2,
            )?;
            draw_simple_text(
                canvas,
                ">",
                (menu_x + 350) as i32,
                item_y as i32,
                sdl2::pixels::Color::RGB(255, 200, 100),
                2,
            )?;
        }
    }

    draw_simple_text(
        canvas,
        "ARROWS   NAVIGATE",
        (menu_x + 40) as i32,
        (menu_y + 245) as i32,
        sdl2::pixels::Color::RGB(140, 140, 150),
        1,
    )?;
    draw_simple_text(
        canvas,
        "SHIFT   10",
        (menu_x + 220) as i32,
        (menu_y + 245) as i32,
        sdl2::pixels::Color::RGB(140, 140, 150),
        1,
    )?;

    Ok(())
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG)?;

    let window_scale = calculate_window_scale(&video_subsystem);
    let window_width = GAME_WIDTH * window_scale;
    let window_height = GAME_HEIGHT * window_scale;

    println!("Monitor scale: {}x (window: {}x{})", window_scale, window_width, window_height);

    let window = video_subsystem
        .window("Game 1 - 8-Directional Character Animation", window_width, window_height)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_logical_size(GAME_WIDTH, GAME_HEIGHT).map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump()?;

    let player_config = AnimationConfig::load_from_file("assets/config/player_animations.json")
        .map_err(|e| format!("Failed to load player animation config: {}", e))?;
    let slime_config = AnimationConfig::load_from_file("assets/config/slime_animations.json")
        .map_err(|e| format!("Failed to load slime animation config: {}", e))?;
    let punch_config = AnimationConfig::load_from_file("assets/config/punch_effect.json")
        .map_err(|e| format!("Failed to load punch effect config: {}", e))?;

    // Create Systems struct to hold configs and helpers
    let mut systems = Systems::new(player_config.clone(), slime_config.clone(), punch_config.clone());

    let character_texture = load_texture(&texture_creator, "assets/sprites/new_player/Character-Base.png")?;
    let slime_texture = load_texture(&texture_creator, "assets/sprites/slime/Slime.png")?;
    let _background_texture = load_texture(&texture_creator, "assets/backgrounds/background_meadow.png")?;
    let punch_texture = load_texture(&texture_creator, "assets/sprites/new_player/punch_effect.png")?;
    let grass_tile_texture = load_texture(&texture_creator, "assets/backgrounds/tileable/grass_tile.png")?;

    let item_registry = ItemRegistry::create_default();
    println!("✓ Item registry initialized");

    let mut item_textures = HashMap::new();
    for item_def in item_registry.all_items() {
        let texture = load_texture(&texture_creator, &item_def.sprite_path)?;
        item_textures.insert(item_def.id.clone(), texture);
    }
    println!("✓ Loaded {} item textures", item_textures.len());

    let save_dir = dirs::home_dir()
        .map(|p| p.join(".game1/saves"))
        .unwrap_or_else(|| std::path::PathBuf::from("./saves"));
    let mut save_manager = SaveManager::new(&save_dir)
        .map_err(|e| format!("Failed to create save manager: {}", e))?;

    let entity_texture = load_texture(&texture_creator, "assets/sprites/the_entity/entity_awaken.png")?;

    let (player, slimes, world_grid, render_grid, entities, player_inventory, dropped_items) =
        match load_game(&save_manager, &player_config, &slime_config, &character_texture, &slime_texture, &entity_texture, &item_textures) {
            Ok((loaded_player, loaded_slimes, loaded_world, loaded_entities, loaded_inventory, loaded_items)) => {
                println!("✓ Loaded existing save!");
                let loaded_render_grid = RenderGrid::new(&loaded_world);
                (loaded_player, loaded_slimes, loaded_world, loaded_render_grid, loaded_entities, loaded_inventory, loaded_items)
            }
            Err(_) => {
                println!("No existing save found, starting new game");

                let animation_controller = player_config.create_controller(
                    &character_texture,
                    &["idle", "running", "attack", "damage", "death"],
                )?;
                let mut new_player = Player::new(300, 200, 32, 32, 3);
                new_player.set_animation_controller(animation_controller);

                let new_world_grid = WorldGrid::new(40, 24, TileId::Grass);
                let new_render_grid = RenderGrid::new(&new_world_grid);

                let entity_spawn_data = [
                    (160, 120, EntityType::Attack),
                    (480, 120, EntityType::Defense),
                    (160, 240, EntityType::Speed),
                    (480, 240, EntityType::Regeneration),
                ];

                let mut new_entities: Vec<TheEntity> = Vec::new();
                for (id, (x, y, entity_type)) in entity_spawn_data.iter().enumerate() {
                    let mut frames = Vec::new();
                    for i in 0..13 {
                        frames.push(sprite::Frame::new(i * 32, 0, 32, 32, 100));
                    }
                    let sprite_sheet = sprite::SpriteSheet::new(&entity_texture, frames);
                    new_entities.push(TheEntity::new(id, *x, *y, *entity_type, sprite_sheet));
                }

                (new_player, Vec::new(), new_world_grid, new_render_grid, new_entities, PlayerInventory::new(), Vec::new())
            }
        };

    // Create GameWorld struct to hold all entities
    let mut world = GameWorld {
        player,
        slimes,
        entities,
        dropped_items,
        world_grid,
        render_grid,
        player_inventory,
        attack_effects: Vec::new(),
        floating_texts: Vec::new(),
        active_attack: None,
    };

    let mut game_state = GameState::Playing;



    let player_health_bar = HealthBar::new();
    let enemy_health_bar = HealthBar::with_style(HealthBarStyle {
        health_color: Color::RGB(150, 0, 150),
        low_health_color: Color::RGB(200, 0, 0),
        ..Default::default()
    });
    let floating_text_renderer = FloatingText::new();
    let buff_display = BuffDisplay::new(&texture_creator)?;

    let save_exit_menu = SaveExitMenu::new();
    let death_screen = DeathScreen::new();
    let inventory_ui = InventoryUI::new(&item_textures, &item_registry);

    // Create UIManager struct to hold all UI state
    let mut ui = UIManager {
        save_exit_menu,
        death_screen,
        inventory_ui,
        player_health_bar,
        enemy_health_bar,
        floating_text_renderer,
        buff_display,
        debug_menu_state: DebugMenuState::Closed,
        show_collision_boxes: false,
        show_tile_grid: false,
        is_tilling: false,
        last_tilled_tile: None,
        mouse_x: 0,
        mouse_y: 0,
    };



    println!("Controls:");
    println!("WASD - Move player");
    println!("M Key - Attack");
    println!("F3 - Debug Stats Menu (adjust combat values!)");
    println!("F5 - Quick Save");
    println!("F9 - Load Game");
    println!("ESC - Exit Menu (Save & Exit, Exit Without Saving, Cancel)");
    println!("B Key - Toggle collision debug boxes");
    println!("G Key - Toggle tile grid debug view");
    println!("1 Key - Select Grass tile");
    println!("2 Key - Select Dirt tile");
    println!("Left Click - Place selected tile");
    println!("Right Click - Spawn slime");
    println!("\n=== NEW: Tile Placement System ===");
    println!("- Select tiles with 1 (Grass) or 2 (Dirt)");
    println!("- Left click to place tiles in the world");
    println!("- Right click to spawn slimes for testing");
    println!("\n=== Collision System ===");
    println!("- Push-apart physics prevents overlap");
    println!("- Touching slimes without attacking damages player (10 HP total)");
    println!("- 1 second invulnerability after taking damage");



        'running: loop {
        // PHASE 1: Handle input events
        if handle_events(
            &mut event_pump,
            &mut game_state,
            &mut world,
            &mut systems,
            &mut ui,
            &mut save_manager,
            &character_texture,
            &slime_texture,
            &entity_texture,
            &punch_texture,
            &item_textures,
            &item_registry,
            &mut canvas,
        )? {
            break 'running; // Quit requested
        }

        let is_ui_active = ui.inventory_ui.is_open ||
                           matches!(ui.debug_menu_state, DebugMenuState::Open { .. }) ||
                           game_state == GameState::ExitMenu ||
                           game_state == GameState::Dead;

        // PHASE 2: Update game state
        if game_state == GameState::Playing && !is_ui_active {
            // Check for player death
            if world.player.state.is_dead() {
                game_state = GameState::Dead;
                ui.death_screen.trigger();
                println!("Player died!");
            }

            let keyboard_state_for_player_update = event_pump.keyboard_state();
            update_world(
                &mut world,
                &mut systems,
                &item_textures,
                &item_registry,
                &keyboard_state_for_player_update,
            )?;
        }

        // Handle death screen respawn
        if game_state == GameState::Dead {
            if ui.death_screen.should_respawn() {
                world.player.respawn(GAME_WIDTH as i32 / 2, GAME_HEIGHT as i32 / 2);
                ui.death_screen.reset();
                game_state = GameState::Playing;
                println!("Player respawned!");
            }
        }

        // PHASE 3: Render
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        world.render_grid.render(&mut canvas, &grass_tile_texture)?;

        render_with_depth_sorting(&mut canvas, &world.player, &world.slimes, &systems.static_objects, &world.entities, &world.dropped_items)?;

        for effect in &world.attack_effects {
            effect.render(&mut canvas, SPRITE_SCALE)?;
        }

        if world.player.state.is_alive() {
            // Health bar expects top-left coordinates, but player uses anchor (bottom-center)
            // Calculate top-left from anchor for health bar rendering
            let player_top_left_x = world.player.x - ((world.player.width * SPRITE_SCALE) / 2) as i32;
            let player_top_left_y = world.player.y - (world.player.height * SPRITE_SCALE) as i32;

            ui.player_health_bar.render(
                &mut canvas,
                player_top_left_x,
                player_top_left_y,
                world.player.width * SPRITE_SCALE,
                world.player.height * SPRITE_SCALE,
                world.player.stats.health.percentage(),
            )?;
        }

        for slime in &world.slimes {
            if slime.is_alive {
                // Health bar expects top-left coordinates, but slime uses anchor (bottom-center)
                // Calculate top-left from anchor for health bar rendering
                let slime_top_left_x = slime.x - ((slime.width * SPRITE_SCALE) / 2) as i32;
                let slime_top_left_y = slime.y - (slime.height * SPRITE_SCALE) as i32;

                ui.enemy_health_bar.render(
                    &mut canvas,
                    slime_top_left_x,
                    slime_top_left_y,
                    slime.width * SPRITE_SCALE,
                    slime.height * SPRITE_SCALE,
                    slime.health as f32 / 8.0, // Slimes have max 8 HP
                )?;
            }
        }

        for text in &world.floating_texts {
            let alpha = ((1.0 - text.lifetime / text.max_lifetime) * 255.0) as u8;
            ui.floating_text_renderer.render(
                &mut canvas,
                text.x as i32,
                text.y as i32,
                &text.text,
                text.color,
                alpha,
            )?;
        }

        if game_state == GameState::Playing {
            ui.buff_display.render(
                &mut canvas,
                &world.player.active_modifiers,
                systems.has_regen,
            )?;
        }

        if ui.show_collision_boxes {
            // RED: Environmental collision boxes (for push physics, walls)
            canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 0, 0, 128));

            let player_collision = world.player.get_bounds();
            canvas.draw_rect(player_collision).map_err(|e| e.to_string())?;

            for slime in &world.slimes {
                let slime_bounds = slime.get_bounds();
                canvas.draw_rect(slime_bounds).map_err(|e| e.to_string())?;

                // YELLOW: Show where sprite SHOULD render (anchor visualization)
                canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 255, 0, 200));
                let sprite_render_x = slime.x - ((slime.width * SPRITE_SCALE) / 2) as i32;
                let sprite_render_y = slime.y - (slime.height * SPRITE_SCALE) as i32;
                let sprite_rect = Rect::new(
                    sprite_render_x,
                    sprite_render_y,
                    slime.width * SPRITE_SCALE,
                    slime.height * SPRITE_SCALE
                );
                canvas.draw_rect(sprite_rect).map_err(|e| e.to_string())?;

                // WHITE: Show anchor point
                canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 255, 255, 255));
                let anchor_size: u32 = 4;
                let anchor_rect = Rect::new(
                    slime.x - (anchor_size as i32) / 2,
                    slime.y - (anchor_size as i32) / 2,
                    anchor_size,
                    anchor_size
                );
                canvas.fill_rect(anchor_rect).map_err(|e| e.to_string())?;

                // Restore red color for entities
                canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 0, 0, 128));
            }

            for entity in &world.entities {
                let entity_bounds = entity.get_bounds();
                canvas.draw_rect(entity_bounds).map_err(|e| e.to_string())?;
            }

            // BLUE: Damage hitboxes (for getting hit by enemies)
            canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 100, 255, 128));

            let player_damage = world.player.get_damage_bounds();
            canvas.draw_rect(player_damage).map_err(|e| e.to_string())?;

            // GREEN: Attack hitboxes (for hitting enemies)
            if let Some(ref attack) = world.active_attack {
                canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 255, 0, 128));
                let attack_hitbox = attack.get_hitbox();
                canvas.draw_rect(attack_hitbox).map_err(|e| e.to_string())?;
            }
        }

        if ui.show_tile_grid {
            canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 255, 0, 128));
            for x in 0..=world.world_grid.width {
                let line_x = (x * 32) as i32;
                canvas.draw_line(
                    sdl2::rect::Point::new(line_x, 0),
                    sdl2::rect::Point::new(line_x, GAME_HEIGHT as i32)
                ).map_err(|e| e.to_string())?;
            }
            for y in 0..=world.world_grid.height {
                let line_y = (y * 32) as i32;
                canvas.draw_line(
                    sdl2::rect::Point::new(0, line_y),
                    sdl2::rect::Point::new(GAME_WIDTH as i32, line_y)
                ).map_err(|e| e.to_string())?;
            }

            for y in 0..world.world_grid.height {
                for x in 0..world.world_grid.width {
                    if let Some(tile_id) = world.world_grid.get_tile(x as i32, y as i32) {
                        let color = match tile_id {
                            TileId::Grass => sdl2::pixels::Color::RGB(0, 255, 0),
                            TileId::Dirt => sdl2::pixels::Color::RGB(139, 69, 19),
                        };
                        canvas.set_draw_color(color);
                        let indicator_rect = sdl2::rect::Rect::new(
                            (x * 32 + 12) as i32,
                            (y * 32 + 12) as i32,
                            8,
                            8
                        );
                        canvas.fill_rect(indicator_rect).map_err(|e| e.to_string())?;
                    }
                }
            }
        }

        if false { // Set to true to see debug info
            println!(
                "Player: pos=({}, {}), vel=({}, {}), state={:?}",
                world.player.position().0,
                world.player.position().1,
                world.player.velocity().0,
                world.player.velocity().1,
                world.player.current_animation_state()
            );
        }

        if game_state == GameState::Playing {
            ui.inventory_ui.render(&mut canvas, &world.player_inventory, world.player_inventory.selected_hotbar_slot, ui.mouse_x, ui.mouse_y)?;
        }

        if game_state == GameState::Dead {
            ui.death_screen.render(&mut canvas)?;
        }

        if game_state == GameState::ExitMenu {
            ui.save_exit_menu.render(&mut canvas)?;
        }

        if let DebugMenuState::Open { selected_index } = ui.debug_menu_state {
            render_debug_menu(&mut canvas, &world.player, &systems.debug_config, selected_index)?;
        }

        canvas.present();

        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}