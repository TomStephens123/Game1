use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

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

use animation::AnimationConfig;
use attack_effect::AttackEffect;
use collision::{
    calculate_overlap, check_collisions_with_collection, check_static_collisions, Collidable,
    StaticCollidable, StaticObject,
};
use combat::{DamageEvent, DamageSource};
use dropped_item::DroppedItem;
use gui::{SaveExitMenu, SaveExitOption, DeathScreen, InventoryUI};
use inventory::PlayerInventory;
use item::ItemRegistry;
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
struct FloatingTextInstance {
    x: f32,
    y: f32,
    text: String,
    color: Color,
    lifetime: f32,
    max_lifetime: f32,
}

/// Debug menu state
#[derive(Debug, Clone, Copy, PartialEq)]
enum DebugMenuState {
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
}

impl DebugMenuItem {
    fn all() -> Vec<Self> {
        vec![
            Self::PlayerMaxHealth,
            Self::PlayerAttackDamage,
            Self::PlayerAttackSpeed,
            Self::SlimeHealth,
            Self::SlimeContactDamage,
        ]
    }

    fn name(&self) -> &str {
        match self {
            Self::PlayerMaxHealth => "Player Max HP",
            Self::PlayerAttackDamage => "Player Damage",
            Self::PlayerAttackSpeed => "Player Atk Spd",
            Self::SlimeHealth => "Slime Health",
            Self::SlimeContactDamage => "Slime Contact Dmg",
        }
    }
}

/// Debug configuration for combat tuning
#[derive(Debug, Clone)]
struct DebugConfig {
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

    let (mut player, mut slimes, mut world_grid, mut render_grid, mut entities, mut player_inventory, mut dropped_items) =
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

    let mut game_state = GameState::Playing;

    let mut debug_menu_state = DebugMenuState::Closed;
    let mut debug_config = DebugConfig::new();

    let player_health_bar = HealthBar::new();
    let enemy_health_bar = HealthBar::with_style(HealthBarStyle {
        health_color: Color::RGB(150, 0, 150),
        low_health_color: Color::RGB(200, 0, 0),
        ..Default::default()
    });
    let floating_text_renderer = FloatingText::new();
    let buff_display = BuffDisplay::new(&texture_creator)?;

    let mut save_exit_menu = SaveExitMenu::new();
    let mut death_screen = DeathScreen::new();
    let mut inventory_ui = InventoryUI::new();

    let boundary_thickness = 10;
    let static_objects = vec![
        StaticObject::new(0, -(boundary_thickness as i32), GAME_WIDTH, boundary_thickness),
        StaticObject::new(-(boundary_thickness as i32), 0, boundary_thickness, GAME_HEIGHT),
        StaticObject::new(GAME_WIDTH as i32, 0, boundary_thickness, GAME_HEIGHT),
        StaticObject::new(0, GAME_HEIGHT as i32, GAME_WIDTH, boundary_thickness),
    ];

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

    let mut regen_timer = Instant::now();
    let regen_interval = 5.0;

    let mut floating_texts: Vec<FloatingTextInstance> = Vec::new();

    let mut has_regen = false;

    let mut active_attack: Option<combat::AttackEvent> = None;
    let mut attack_effects: Vec<AttackEffect> = Vec::new();
    let mut selected_tile = TileId::Grass;
    let mut is_painting = false;
    let mut last_painted_tile: Option<(i32, i32)> = None;
    let mut show_collision_boxes = false;
    let mut show_tile_grid = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    match game_state {
                        GameState::Playing => {
                            game_state = GameState::ExitMenu;
                        }
                        GameState::ExitMenu => {
                            game_state = GameState::Playing;
                        }
                        GameState::Dead => {
                            game_state = GameState::ExitMenu;
                            death_screen.reset();
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } if game_state == GameState::ExitMenu => {
                    save_exit_menu.navigate_up();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } if game_state == GameState::ExitMenu => {
                    save_exit_menu.navigate_down();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Return | Keycode::Space),
                    ..
                } if game_state == GameState::ExitMenu => {
                    match save_exit_menu.selected_option() {
                        SaveExitOption::SaveAndExit => {
                            if let Err(e) = save_game(&mut save_manager, &player, &slimes, &world_grid, &entities, &player_inventory, &dropped_items) {
                                eprintln!("Failed to save: {}", e);
                            }
                            break 'running;
                        }
                        SaveExitOption::Cancel => {
                            game_state = GameState::Playing;
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F5),
                    ..
                } if game_state == GameState::Playing => {
                    if let Err(e) = save_game(&mut save_manager, &player, &slimes, &world_grid, &entities, &player_inventory, &dropped_items) {
                        eprintln!("Failed to save: {}", e);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F9),
                    ..
                } if game_state == GameState::Playing => {
                    match load_game(&save_manager, &player_config, &slime_config, &character_texture, &slime_texture, &entity_texture, &item_textures) {
                        Ok((loaded_player, loaded_slimes, loaded_world, loaded_entities, loaded_inventory, loaded_items)) => {
                            player = loaded_player;
                            slimes = loaded_slimes;
                            world_grid = loaded_world;
                            render_grid = RenderGrid::new(&world_grid);
                            entities = loaded_entities;
                            player_inventory = loaded_inventory;
                            dropped_items = loaded_items;
                            attack_effects.clear();
                            active_attack = None;
                            println!("✓ Game loaded successfully!");
                        }
                        Err(e) => {
                            eprintln!("Failed to load: {}", e);
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F3),
                    ..
                } if game_state == GameState::Playing => {
                    debug_menu_state = match debug_menu_state {
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
                } if matches!(debug_menu_state, DebugMenuState::Open { .. }) => {
                    if let DebugMenuState::Open { selected_index } = debug_menu_state {
                        let items = DebugMenuItem::all();
                        let new_index = if selected_index == 0 {
                            items.len() - 1
                        } else {
                            selected_index - 1
                        };
                        debug_menu_state = DebugMenuState::Open { selected_index: new_index };
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } if matches!(debug_menu_state, DebugMenuState::Open { .. }) => {
                    if let DebugMenuState::Open { selected_index } = debug_menu_state {
                        let items = DebugMenuItem::all();
                        let new_index = (selected_index + 1) % items.len();
                        debug_menu_state = DebugMenuState::Open { selected_index: new_index };
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    keymod,
                    ..
                } if matches!(debug_menu_state, DebugMenuState::Open { .. }) => {
                    if let DebugMenuState::Open { selected_index } = debug_menu_state {
                        let items = DebugMenuItem::all();
                        let item = items[selected_index];
                        let shift_held = keymod.intersects(sdl2::keyboard::Mod::LSHIFTMOD | sdl2::keyboard::Mod::RSHIFTMOD);
                        let delta = if shift_held { -10.0 } else { -1.0 };

                        match item {
                            DebugMenuItem::PlayerMaxHealth => {
                                let new_val = (player.stats.max_health + delta).max(1.0);
                                player.stats.max_health = new_val;
                                player.stats.health.set_max(new_val);
                            }
                            DebugMenuItem::PlayerAttackDamage => {
                                player.stats.attack_damage = (player.stats.attack_damage + delta).max(0.0);
                            }
                            DebugMenuItem::PlayerAttackSpeed => {
                                player.stats.attack_speed = (player.stats.attack_speed + delta).max(0.1);
                            }
                            DebugMenuItem::SlimeHealth => {
                                debug_config.slime_base_health = (debug_config.slime_base_health as f32 + delta).max(1.0) as i32;
                            }
                            DebugMenuItem::SlimeContactDamage => {
                                debug_config.slime_contact_damage = (debug_config.slime_contact_damage + delta).max(0.0);
                            }
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    keymod,
                    ..
                } if matches!(debug_menu_state, DebugMenuState::Open { .. }) => {
                    if let DebugMenuState::Open { selected_index } = debug_menu_state {
                        let items = DebugMenuItem::all();
                        let item = items[selected_index];
                        let shift_held = keymod.intersects(sdl2::keyboard::Mod::LSHIFTMOD | sdl2::keyboard::Mod::RSHIFTMOD);
                        let delta = if shift_held { 10.0 } else { 1.0 };

                        match item {
                            DebugMenuItem::PlayerMaxHealth => {
                                let new_val = player.stats.max_health + delta;
                                player.stats.max_health = new_val;
                                player.stats.health.set_max(new_val);
                            }
                            DebugMenuItem::PlayerAttackDamage => {
                                player.stats.attack_damage += delta;
                            }
                            DebugMenuItem::PlayerAttackSpeed => {
                                player.stats.attack_speed += delta;
                            }
                            DebugMenuItem::SlimeHealth => {
                                debug_config.slime_base_health = (debug_config.slime_base_health as f32 + delta) as i32;
                            }
                            DebugMenuItem::SlimeContactDamage => {
                                debug_config.slime_contact_damage += delta;
                            }
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::M),
                    ..
                } => {
                    if let Some(attack_event) = player.start_attack() {
                        active_attack = Some(attack_event.clone());

                        let offset = 20;
                        let (offset_x, offset_y) = match player.direction {
                            crate::animation::Direction::North => (0, -offset),
                            crate::animation::Direction::NorthEast => (offset, -offset),
                            crate::animation::Direction::East => (offset, 0),
                            crate::animation::Direction::SouthEast => (offset, offset),
                            crate::animation::Direction::South => (0, offset),
                            crate::animation::Direction::SouthWest => (-offset, offset),
                            crate::animation::Direction::West => (-offset, 0),
                            crate::animation::Direction::NorthWest => (-offset, -offset),
                        };

                        let effect_x = player.x + offset_x;
                        let effect_y = player.y + offset_y;

                        match punch_config.create_controller(&punch_texture, &["punch"]) {
                            Ok(punch_animation_controller) => {
                                attack_effects.push(AttackEffect::new(
                                    effect_x,
                                    effect_y,
                                    32,
                                    32,
                                    player.direction,
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
                } => {
                    show_collision_boxes = !show_collision_boxes;
                    println!("Collision boxes: {}", if show_collision_boxes { "ON" } else { "OFF" });
                }
                Event::KeyDown {
                    keycode: Some(Keycode::G),
                    ..
                } => {
                    show_tile_grid = !show_tile_grid;
                    println!("Tile grid debug: {}", if show_tile_grid { "ON" } else { "OFF" });
                }
                Event::KeyDown {
                    keycode: Some(Keycode::I),
                    ..
                } => {
                    inventory_ui.toggle();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => {
                    selected_tile = TileId::Grass;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => {
                    selected_tile = TileId::Dirt;
                }
                Event::MouseButtonDown { mouse_btn: sdl2::mouse::MouseButton::Left, x, y, .. } => {
                    is_painting = true;
                    let tile_x = x / 32;
                    let tile_y = y / 32;

                    if world_grid.set_tile(tile_x, tile_y, selected_tile) {
                        render_grid.update_tile_and_neighbors(&world_grid, tile_x, tile_y);
                        last_painted_tile = Some((tile_x, tile_y));
                    }
                }
                Event::MouseButtonUp { mouse_btn: sdl2::mouse::MouseButton::Left, .. } => {
                    is_painting = false;
                    last_painted_tile = None;
                }
                Event::MouseMotion { x, y, .. } => {
                    if is_painting {
                        let tile_x = x / 32;
                        let tile_y = y / 32;

                        if last_painted_tile != Some((tile_x, tile_y)) {
                            if world_grid.set_tile(tile_x, tile_y, selected_tile) {
                                render_grid.update_tile_and_neighbors(&world_grid, tile_x, tile_y);
                                last_painted_tile = Some((tile_x, tile_y));
                            }
                        }
                    }
                }
                Event::MouseButtonDown { mouse_btn: sdl2::mouse::MouseButton::Right, x, y, .. } => {
                    let slime_animation_controller = slime_config.create_controller(
                        &slime_texture,
                        &["slime_idle", "jump", "slime_damage", "slime_death"],
                    )?;
                    let center_offset = (32 * SPRITE_SCALE / 2) as i32;
                    let mut new_slime = Slime::new(x - center_offset, y - center_offset, slime_animation_controller);
                    new_slime.health = debug_config.slime_base_health;
                    slimes.push(new_slime);
                }
                _ => {} // Ignore other events
            }
        }

        if game_state == GameState::Playing {
            if player.state.is_dead() {
                game_state = GameState::Dead;
                death_screen.trigger();
                println!("Player died!");
            }

            let keyboard_state = event_pump.keyboard_state();
            player.update(&keyboard_state);

            if let Some(ref attack) = active_attack {
                let attack_hitbox = attack.get_hitbox();

                for slime in &mut slimes {
                    let slime_bounds = slime.get_bounds();

                    if collision::aabb_intersect(&attack_hitbox, &slime_bounds) {
                        slime.take_damage(attack.damage as i32);
                    }
                }

                for entity in &mut entities {
                    entity.check_hit(&attack_hitbox);
                }

                active_attack = None;
            }

            for slime in &mut slimes {
                slime.update();
            }

            let delta_time = 1.0 / 60.0;
            for entity in &mut entities {
                entity.update(delta_time);
            }

            for text in &mut floating_texts {
                text.lifetime += delta_time;
                text.y -= 20.0 * delta_time;
            }
            floating_texts.retain(|text| text.lifetime < text.max_lifetime);

            player.active_modifiers.clear();
            has_regen = false;
            for entity in &entities {
                if entity.state == EntityState::Awake {
                    match entity.entity_type {
                        EntityType::Attack => {
                            player.active_modifiers.push(ModifierEffect {
                                stat_type: StatType::AttackDamage,
                                modifier: StatModifier::Flat(1.0),
                                duration: None,
                                source: "Pyramid of Attack".to_string(),
                            });
                        }
                        EntityType::Defense => {
                            player.active_modifiers.push(ModifierEffect {
                                stat_type: StatType::Defense,
                                modifier: StatModifier::Flat(1.0),
                                duration: None,
                                source: "Pyramid of Defense".to_string(),
                            });
                        }
                        EntityType::Speed => {
                            player.active_modifiers.push(ModifierEffect {
                                stat_type: StatType::MovementSpeed,
                                modifier: StatModifier::Flat(1.0),
                                duration: None,
                                source: "Pyramid of Speed".to_string(),
                            });
                        }
                        EntityType::Regeneration => {
                            has_regen = true;
                        }
                    }
                }
            }

            if has_regen && regen_timer.elapsed().as_secs_f32() >= regen_interval {
                if player.stats.health.current() < player.stats.max_health {
                    player.stats.health.heal(2.0);

                    floating_texts.push(FloatingTextInstance {
                        x: player.x as f32 + (player.width * SPRITE_SCALE) as f32 / 2.0,
                        y: player.y as f32,
                        text: "+2".to_string(),
                        color: Color::RGB(0, 255, 0),
                        lifetime: 0.0,
                        max_lifetime: 1.5,
                    });

                    for entity in &entities {
                        if entity.state == EntityState::Awake && entity.entity_type == EntityType::Regeneration {
                            floating_texts.push(FloatingTextInstance {
                                x: entity.x as f32 + 16.0,
                                y: entity.y as f32,
                                text: "+2".to_string(),
                                color: Color::RGB(0, 255, 0),
                                lifetime: 0.0,
                                max_lifetime: 1.5,
                            });
                            break; // Only one regen entity
                        }
                    }
                }
                regen_timer = Instant::now();
            }
        }

        if game_state == GameState::Dead {
            if death_screen.should_respawn() {
                player.respawn(GAME_WIDTH as i32 / 2, GAME_HEIGHT as i32 / 2);
                death_screen.reset();
                game_state = GameState::Playing;
                println!("Player respawned!");
            }
        }

        for effect in &mut attack_effects {
            effect.update();
        }

        attack_effects.retain(|effect| !effect.is_finished());

        let colliding_slime_indices = check_collisions_with_collection(&player, &slimes);

        for slime_index in colliding_slime_indices {
            let player_bounds = player.get_bounds();
            let slime_bounds = slimes[slime_index].get_bounds();

            let (overlap_x, overlap_y) = calculate_overlap(&player_bounds, &slime_bounds);

            if overlap_x.abs() < overlap_y.abs() {
                player.apply_push(-overlap_x * 3 / 10, 0);
                slimes[slime_index].apply_push(overlap_x * 7 / 10, 0);
            } else {
                player.apply_push(0, -overlap_y * 3 / 10);
                slimes[slime_index].apply_push(0, overlap_y * 7 / 10);
            }

            if !player.is_attacking && !slimes[slime_index].is_invulnerable() {
                let damage = DamageEvent::physical(debug_config.slime_contact_damage, DamageSource::Enemy);
                let damage_result = player.take_damage(damage);
                if damage_result.is_fatal {
                    // Drop all items from player inventory
                    for item_stack_option in player_inventory.inventory.slots.iter_mut() {
                        if let Some(item_stack) = item_stack_option.take() {
                            let mut item_animation_controller = animation::AnimationController::new();
                            let item_frames = vec![
                                sprite::Frame::new(0, 0, 32, 32, 300),
                            ];
                            let item_texture = item_textures.get(&item_stack.item_id).ok_or(format!("Missing texture for item {}", item_stack.item_id))?;
                            let item_sprite_sheet = SpriteSheet::new(item_texture, item_frames);
                            item_animation_controller.add_animation("item_idle".to_string(), item_sprite_sheet);
                            item_animation_controller.set_state("item_idle".to_string());

                            let dropped_item = DroppedItem::new(
                                player.x,
                                player.y,
                                item_stack.item_id.clone(),
                                item_stack.quantity,
                                item_animation_controller,
                            );
                            dropped_items.push(dropped_item);
                        }
                    }
                    println!("Player died and dropped all items.");
                }
            }
        }

        for slime in &mut slimes {
            if slime.is_dying() && !slime.has_dropped_loot {
                slime.has_dropped_loot = true;
                let mut item_animation_controller = animation::AnimationController::new();

                let item_frames = vec![
                    sprite::Frame::new(0, 0, 32, 32, 300),
                ];

                let item_texture = item_textures.get("slime_ball").ok_or("Missing slime_ball texture in item_textures map")?;

                let item_sprite_sheet = SpriteSheet::new(item_texture, item_frames);

                item_animation_controller.add_animation("item_idle".to_string(), item_sprite_sheet);
                item_animation_controller.set_state("item_idle".to_string());

                let dropped_item = DroppedItem::new(
                    slime.x + 32,
                    slime.y + 32,
                    "slime_ball".to_string(),
                    1,
                    item_animation_controller,
                );

                dropped_items.push(dropped_item);
                println!("Slime dropped slime_ball at ({}, {})", slime.x + 32, slime.y + 32);
            }
        }

        slimes.retain(|slime| slime.is_alive);

        let player_bounds = player.get_bounds();
        dropped_items.retain(|item| {
            if !item.can_pickup {
                return true; // Keep items in cooldown
            }

            if player_bounds.has_intersection(item.get_bounds()) {
                match player_inventory.quick_add(&item.item_id, item.quantity, &item_registry) {
                    Ok(overflow) => {
                        if overflow == 0 {
                            // Fully picked up
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

        dropped_items.retain_mut(|item| !item.update()); // Remove if despawned

        let mut all_static_collidables: Vec<&dyn StaticCollidable> = Vec::new();
        for obj in &static_objects {
            all_static_collidables.push(obj);
        }
        for entity in &entities {
            all_static_collidables.push(entity);
        }

        let static_collisions = check_static_collisions(&player, &all_static_collidables);

        for obj_index in static_collisions {
            let player_bounds = player.get_bounds();
            let obj_bounds = all_static_collidables[obj_index].get_bounds();

            let (overlap_x, overlap_y) = calculate_overlap(&player_bounds, &obj_bounds);

            // Push player out completely (100% push on player, 0% on static object)
            if overlap_x.abs() < overlap_y.abs() {
                // Push on X axis
                player.apply_push(-overlap_x, 0);
            } else {
                // Push on Y axis
                player.apply_push(0, -overlap_y);
            }
        }

        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        render_grid.render(&mut canvas, &grass_tile_texture)?;

        render_with_depth_sorting(&mut canvas, &player, &slimes, &static_objects, &entities, &dropped_items)?;

        for effect in &attack_effects {
            effect.render(&mut canvas, SPRITE_SCALE)?;
        }

        if player.state.is_alive() {
            player_health_bar.render(
                &mut canvas,
                player.x,
                player.y,
                player.width * SPRITE_SCALE,
                player.height * SPRITE_SCALE,
                player.stats.health.percentage(),
            )?;
        }

        for slime in &slimes {
            if slime.is_alive {
                enemy_health_bar.render(
                    &mut canvas,
                    slime.x,
                    slime.y,
                    slime.width * SPRITE_SCALE,
                    slime.height * SPRITE_SCALE,
                    slime.health as f32 / 8.0, // Slimes have max 8 HP
                )?;
            }
        }

        for text in &floating_texts {
            let alpha = ((1.0 - text.lifetime / text.max_lifetime) * 255.0) as u8;
            floating_text_renderer.render(
                &mut canvas,
                text.x as i32,
                text.y as i32,
                &text.text,
                text.color,
                alpha,
            )?;
        }

        if game_state == GameState::Playing {
            buff_display.render(
                &mut canvas,
                &player.active_modifiers,
                has_regen,
            )?;
        }

        if show_collision_boxes {
            canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 0, 0, 128));

            let player_bounds = player.get_bounds();
            canvas.draw_rect(player_bounds).map_err(|e| e.to_string())?;

            for slime in &slimes {
                let slime_bounds = slime.get_bounds();
                canvas.draw_rect(slime_bounds).map_err(|e| e.to_string())?;
            }

            for entity in &entities {
                let entity_bounds = entity.get_bounds();
                canvas.draw_rect(entity_bounds).map_err(|e| e.to_string())?;
            }

            if let Some(ref attack) = active_attack {
                canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 255, 0, 128));
                let attack_hitbox = attack.get_hitbox();
                canvas.draw_rect(attack_hitbox).map_err(|e| e.to_string())?;
            }
        }

        if show_tile_grid {
            canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 255, 0, 128));
            for x in 0..=world_grid.width {
                let line_x = (x * 32) as i32;
                canvas.draw_line(
                    sdl2::rect::Point::new(line_x, 0),
                    sdl2::rect::Point::new(line_x, GAME_HEIGHT as i32)
                ).map_err(|e| e.to_string())?;
            }
            for y in 0..=world_grid.height {
                let line_y = (y * 32) as i32;
                canvas.draw_line(
                    sdl2::rect::Point::new(0, line_y),
                    sdl2::rect::Point::new(GAME_WIDTH as i32, line_y)
                ).map_err(|e| e.to_string())?;
            }

            for y in 0..world_grid.height {
                for x in 0..world_grid.width {
                    if let Some(tile_id) = world_grid.get_tile(x as i32, y as i32) {
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
                player.position().0,
                player.position().1,
                player.velocity().0,
                player.velocity().1,
                player.current_animation_state()
            );
        }

        if game_state == GameState::Playing {
            inventory_ui.render(&mut canvas, &player_inventory, &item_registry, &item_textures, player_inventory.selected_hotbar_slot)?;
        }

        if game_state == GameState::Dead {
            death_screen.render(&mut canvas)?;
        }

        if game_state == GameState::ExitMenu {
            save_exit_menu.render(&mut canvas)?;
        }

        if let DebugMenuState::Open { selected_index } = debug_menu_state {
            render_debug_menu(&mut canvas, &player, &debug_config, selected_index)?;
        }

        canvas.present();

        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}