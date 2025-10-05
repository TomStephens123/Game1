use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;

mod animation;
mod attack_effect;
mod collision;
mod combat;
mod player;
mod save;
mod slime;
mod sprite;
mod stats;
mod tile;

use animation::AnimationConfig;
use attack_effect::AttackEffect;
use collision::{
    calculate_overlap, check_collisions_with_collection, check_static_collisions, Collidable,
    StaticCollidable, StaticObject,
};
use combat::{DamageEvent, DamageSource};
use player::Player;
use save::{SaveManager, SaveFile, SaveMetadata, SaveType, WorldSaveData, EntitySaveData, Saveable, SaveData, CURRENT_SAVE_VERSION};
use slime::Slime;
use tile::{TileId, TileRegistry, TileType, WorldGrid, RenderGrid};
use std::time::SystemTime;

// Game resolution constants
const GAME_WIDTH: u32 = 640;
const GAME_HEIGHT: u32 = 360;
const SPRITE_SCALE: u32 = 2;

/// Game state for menu/gameplay tracking
#[derive(Debug, Clone, PartialEq)]
enum GameState {
    Playing,
    ExitMenu,
}

/// Exit menu options
#[derive(Debug, Clone, Copy, PartialEq)]
enum ExitMenuOption {
    SaveAndExit,
    Cancel,
}


fn load_character_texture(
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) -> Result<sdl2::render::Texture<'_>, String> {
    texture_creator
        .load_texture("assets/sprites/new_player/Character-Base.png")
        .map_err(|e| format!("Failed to load Character-Base.png: {}", e))
}

fn load_slime_texture(
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) -> Result<sdl2::render::Texture<'_>, String> {
    texture_creator
        .load_texture("assets/sprites/slime/Slime.png")
        .map_err(|e| format!("Failed to load Slime.png: {}", e))
}

fn load_background_texture(
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) -> Result<sdl2::render::Texture<'_>, String> {
    texture_creator
        .load_texture("assets/backgrounds/background_meadow.png")
        .map_err(|e| format!("Failed to load background_meadow.png: {}", e))
}

fn load_grass_tile_texture(
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) -> Result<sdl2::render::Texture<'_>, String> {
    texture_creator
        .load_texture("assets/backgrounds/tileable/grass_tile.png")
        .map_err(|e| format!("Failed to load grass_tile.png: {}", e))
}

fn load_punch_texture(
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) -> Result<sdl2::render::Texture<'_>, String> {
    texture_creator
        .load_texture("assets/sprites/new_player/punch_effect.png")
        .map_err(|e| format!("Failed to load punch_effect.png: {}", e))
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
) -> Result<(Player<'a>, Vec<Slime<'a>>, WorldGrid), String> {
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

    for entity_data in save_file.entities {
        match entity_data.entity_type.as_str() {
            "player" => {
                // Deserialize player
                let save_data = SaveData {
                    data_type: "player".to_string(),
                    json_data: entity_data.data,
                };

                let mut loaded_player = Player::from_save_data(&save_data)
                    .map_err(|e| format!("Failed to load player: {}", e))?;

                // Set up animation controller with textures
                let animation_controller = player_config.create_controller(
                    character_texture,
                    &["idle", "running", "attack"],
                ).map_err(|e| format!("Failed to create player animations: {}", e))?;

                loaded_player.set_animation_controller(animation_controller);
                player = Some(loaded_player);
                println!("  - Loaded player at ({}, {})", entity_data.position.0, entity_data.position.1);
            }
            "slime" => {
                // Deserialize slime
                let save_data = SaveData {
                    data_type: "slime".to_string(),
                    json_data: entity_data.data,
                };

                let mut loaded_slime = Slime::from_save_data(&save_data)
                    .map_err(|e| format!("Failed to load slime: {}", e))?;

                // Set up animation controller with textures
                let slime_animation_controller = slime_config.create_controller(
                    slime_texture,
                    &["slime_idle", "jump"],
                ).map_err(|e| format!("Failed to create slime animations: {}", e))?;

                loaded_slime.set_animation_controller(slime_animation_controller);
                slimes.push(loaded_slime);
            }
            unknown => {
                eprintln!("Warning: Unknown entity type '{}', skipping", unknown);
            }
        }
    }

    let player = player.ok_or_else(|| "No player found in save file".to_string())?;
    println!("  - Loaded {} slimes", slimes.len());
    println!("✓ Game loaded successfully!");

    Ok((player, slimes, world_grid))
}

/// Save the current game state
fn save_game(
    save_manager: &mut SaveManager,
    player: &Player,
    slimes: &[Slime],
    world_grid: &WorldGrid,
) -> Result<(), String> {
    // Collect entity save data
    let mut entities = Vec::new();

    // Save player (entity_id = 0)
    let player_save_data = player.to_save_data()
        .map_err(|e| format!("Failed to save player: {}", e))?;

    entities.push(EntitySaveData {
        entity_id: 0,
        entity_type: "player".to_string(),
        position: player.position(),
        data: player_save_data.json_data,
    });

    // Save slimes (entity_id starts from 1)
    for (i, slime) in slimes.iter().enumerate() {
        let slime_save_data = slime.to_save_data()
            .map_err(|e| format!("Failed to save slime {}: {}", i, e))?;

        entities.push(EntitySaveData {
            entity_id: (i + 1) as u64,
            entity_type: "slime".to_string(),
            position: (slime.x, slime.y),
            data: slime_save_data.json_data,
        });
    }

    // Create world save data
    let world_state = WorldSaveData {
        width: world_grid.width,
        height: world_grid.height,
        tiles: world_grid.to_save_data(),
    };

    // Save counts for debug output
    let entity_count = entities.len();
    let slime_count = entities.len() - 1;
    let world_width = world_state.width;
    let world_height = world_state.height;

    // Create save file
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
        entities,
    };

    // Save to file
    save_manager.save_game(&save_file)
        .map_err(|e| format!("Save failed: {}", e))?;

    println!("✓ Game saved successfully!");
    println!("  - Saved {} entities ({} slimes)", entity_count, slime_count);
    println!("  - Saved world: {}x{} tiles", world_width, world_height);
    Ok(())
}

/// Simple helper to draw text using a basic 5x7 bitmap font
fn draw_simple_text(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    text: &str,
    x: i32,
    y: i32,
    color: sdl2::pixels::Color,
    scale: u32,
) -> Result<(), String> {
    canvas.set_draw_color(color);

    let char_width = 6 * scale;  // 5 pixels + 1 spacing
    let pixel_size = scale as i32;

    for (i, c) in text.chars().enumerate() {
        let char_x = x + (i as i32 * char_width as i32);

        // 5x7 bitmap font patterns (1 = pixel on, 0 = pixel off)
        let pattern: &[u8] = match c.to_ascii_uppercase() {
            'A' => &[0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'C' => &[0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
            'D' => &[0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
            'E' => &[0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
            'G' => &[0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
            'I' => &[0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111],
            'L' => &[0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
            'N' => &[0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
            'O' => &[0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'S' => &[0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
            'T' => &[0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
            'V' => &[0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
            'X' => &[0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
            ' ' => &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
            _ => &[0b11111, 0b11111, 0b11111, 0b11111, 0b11111, 0b11111, 0b11111], // Full block for unknown
        };

        // Draw the character pixel by pixel
        for (row, &pattern_row) in pattern.iter().enumerate() {
            for col in 0..5 {
                if (pattern_row >> (4 - col)) & 1 == 1 {
                    canvas.fill_rect(sdl2::rect::Rect::new(
                        char_x + (col * pixel_size),
                        y + (row as i32 * pixel_size),
                        scale,
                        scale,
                    ))?;
                }
            }
        }
    }

    Ok(())
}

/// Render the exit menu overlay
fn render_exit_menu(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    selected_option: ExitMenuOption,
) -> Result<(), String> {
    // Semi-transparent overlay (need to enable blend mode)
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 180));
    canvas.fill_rect(None)?;
    canvas.set_blend_mode(sdl2::render::BlendMode::None);

    // Draw menu box
    let menu_width = 500;
    let menu_height = 240;
    let menu_x = (GAME_WIDTH - menu_width) / 2;
    let menu_y = (GAME_HEIGHT - menu_height) / 2;

    // Menu background
    canvas.set_draw_color(sdl2::pixels::Color::RGB(30, 30, 40));
    canvas.fill_rect(sdl2::rect::Rect::new(
        menu_x as i32,
        menu_y as i32,
        menu_width,
        menu_height,
    ))?;

    // Menu border (double border for better visibility)
    canvas.set_draw_color(sdl2::pixels::Color::RGB(100, 100, 120));
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

    // Title
    draw_simple_text(
        canvas,
        "EXIT",
        (menu_x + 210) as i32,
        (menu_y + 30) as i32,
        sdl2::pixels::Color::RGB(220, 220, 240),
        3,
    )?;

    // Option positions
    let option_height = 60;
    let option_start_y = menu_y + 100;

    // Define options (only 2 options now)
    let options = [
        ("SAVE AND EXIT", ExitMenuOption::SaveAndExit),
        ("CANCEL", ExitMenuOption::Cancel),
    ];

    for (i, (text, option)) in options.iter().enumerate() {
        let option_y = option_start_y + (i as u32 * option_height);
        let is_selected = *option == selected_option;

        // Draw selection highlight
        if is_selected {
            canvas.set_draw_color(sdl2::pixels::Color::RGB(80, 100, 140));
            canvas.fill_rect(sdl2::rect::Rect::new(
                (menu_x + 15) as i32,
                option_y as i32 - 3,
                menu_width - 30,
                36,
            ))?;
        }

        // Draw option text (centered, larger)
        let text_color = if is_selected {
            sdl2::pixels::Color::RGB(255, 255, 255)
        } else {
            sdl2::pixels::Color::RGB(160, 160, 170)
        };

        draw_simple_text(
            canvas,
            text,
            (menu_x + 80) as i32,
            option_y as i32,
            text_color,
            3,
        )?;
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG)?;

    // Calculate window scale based on monitor size
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

    // Set logical size for automatic pixel-perfect scaling
    canvas.set_logical_size(GAME_WIDTH, GAME_HEIGHT).map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump()?;

    // Load animation configurations
    let player_config = AnimationConfig::load_from_file("assets/config/player_animations.json")
        .map_err(|e| format!("Failed to load player animation config: {}", e))?;
    let slime_config = AnimationConfig::load_from_file("assets/config/slime_animations.json")
        .map_err(|e| format!("Failed to load slime animation config: {}", e))?;
    let punch_config = AnimationConfig::load_from_file("assets/config/punch_effect.json")
        .map_err(|e| format!("Failed to load punch effect config: {}", e))?;

    // Animation system loaded successfully

    // Load sprite textures
    let character_texture = load_character_texture(&texture_creator)?;
    let slime_texture = load_slime_texture(&texture_creator)?;
    let _background_texture = load_background_texture(&texture_creator)?;
    let punch_texture = load_punch_texture(&texture_creator)?;
    let grass_tile_texture = load_grass_tile_texture(&texture_creator)?;

    // Setup tile system - Create and register tile types
    let mut _tile_registry = TileRegistry::new();
    _tile_registry.register(TileType {
        id: TileId::Grass,
        name: "Grass".to_string(),
        tile_size: 32,
    });
    _tile_registry.register(TileType {
        id: TileId::Dirt,
        name: "Dirt".to_string(),
        tile_size: 32,
    });

    // Tile placement state
    let mut selected_tile = TileId::Grass;
    let mut is_painting = false; // Track if mouse button is held down
    let mut last_painted_tile: Option<(i32, i32)> = None; // Prevent painting same tile multiple times

    // Vector to store slimes spawned by mouse clicks
    let mut slimes: Vec<Slime> = Vec::new();

    // Vector to store active attack effects (punch visuals)
    let mut attack_effects: Vec<AttackEffect> = Vec::new();

    // Store the current attack event for hit detection
    let mut active_attack: Option<combat::AttackEvent> = None;

    // Debug toggle for collision visualization
    let mut show_collision_boxes = false;

    // Debug toggle for tile grid visualization
    let mut show_tile_grid = false;

    // Save system
    let save_dir = dirs::home_dir()
        .map(|p| p.join(".game1/saves"))
        .unwrap_or_else(|| std::path::PathBuf::from("./saves"));
    let mut save_manager = SaveManager::new(&save_dir)
        .map_err(|e| format!("Failed to create save manager: {}", e))?;

    // Try to load existing save on startup
    let (mut player, mut slimes, mut world_grid, mut render_grid) =
        match load_game(&save_manager, &player_config, &slime_config, &character_texture, &slime_texture) {
            Ok((loaded_player, loaded_slimes, loaded_world)) => {
                println!("✓ Loaded existing save!");
                let loaded_render_grid = RenderGrid::new(&loaded_world);
                (loaded_player, loaded_slimes, loaded_world, loaded_render_grid)
            }
            Err(_) => {
                // No save exists, create new game
                println!("No existing save found, starting new game");

                // Setup player with animations
                let animation_controller = player_config.create_controller(
                    &character_texture,
                    &["idle", "running", "attack"],
                )?;
                let mut new_player = Player::new(300, 200, 32, 32, 3);
                new_player.set_animation_controller(animation_controller);

                // Create world grid (40x24 tiles, default to grass)
                let new_world_grid = WorldGrid::new(40, 24, TileId::Grass);
                let new_render_grid = RenderGrid::new(&new_world_grid);

                (new_player, Vec::new(), new_world_grid, new_render_grid)
            }
        };

    // Game state for menu handling
    let mut game_state = GameState::Playing;
    let mut exit_menu_selection = ExitMenuOption::SaveAndExit;

    // Window boundary collision - invisible walls at screen edges
    // Made 10px thick to reliably catch player hitbox
    let boundary_thickness = 10;
    let static_objects = vec![
        // Top boundary
        StaticObject::new(0, -(boundary_thickness as i32), GAME_WIDTH, boundary_thickness),
        // Left boundary
        StaticObject::new(-(boundary_thickness as i32), 0, boundary_thickness, GAME_HEIGHT),
        // Right boundary
        StaticObject::new(GAME_WIDTH as i32, 0, boundary_thickness, GAME_HEIGHT),
        // Bottom boundary
        StaticObject::new(0, GAME_HEIGHT as i32, GAME_WIDTH, boundary_thickness),
    ];

    println!("Controls:");
    println!("WASD - Move player");
    println!("M Key - Attack");
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
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    match game_state {
                        GameState::Playing => {
                            // Open exit menu
                            game_state = GameState::ExitMenu;
                            exit_menu_selection = ExitMenuOption::SaveAndExit;
                        }
                        GameState::ExitMenu => {
                            // Close menu (Cancel)
                            game_state = GameState::Playing;
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } if game_state == GameState::ExitMenu => {
                    exit_menu_selection = match exit_menu_selection {
                        ExitMenuOption::SaveAndExit => ExitMenuOption::Cancel,
                        ExitMenuOption::Cancel => ExitMenuOption::SaveAndExit,
                    };
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } if game_state == GameState::ExitMenu => {
                    exit_menu_selection = match exit_menu_selection {
                        ExitMenuOption::SaveAndExit => ExitMenuOption::Cancel,
                        ExitMenuOption::Cancel => ExitMenuOption::SaveAndExit,
                    };
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Return | Keycode::Space),
                    ..
                } if game_state == GameState::ExitMenu => {
                    match exit_menu_selection {
                        ExitMenuOption::SaveAndExit => {
                            if let Err(e) = save_game(&mut save_manager, &player, &slimes, &world_grid) {
                                eprintln!("Failed to save: {}", e);
                            }
                            break 'running;
                        }
                        ExitMenuOption::Cancel => {
                            game_state = GameState::Playing;
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F5),
                    ..
                } if game_state == GameState::Playing => {
                    // Quick save
                    if let Err(e) = save_game(&mut save_manager, &player, &slimes, &world_grid) {
                        eprintln!("Failed to save: {}", e);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F9),
                    ..
                } if game_state == GameState::Playing => {
                    // Load game
                    match load_game(&save_manager, &player_config, &slime_config, &character_texture, &slime_texture) {
                        Ok((loaded_player, loaded_slimes, loaded_world)) => {
                            player = loaded_player;
                            slimes = loaded_slimes;
                            world_grid = loaded_world;
                            render_grid = RenderGrid::new(&world_grid);
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
                    keycode: Some(Keycode::M),
                    ..
                } => {
                    // Attempt to attack (returns None if on cooldown)
                    if let Some(attack_event) = player.start_attack() {
                        // Store attack for hit detection
                        active_attack = Some(attack_event.clone());

                        // Spawn punch effect in front of player based on their direction
                        // Calculate offset to position punch effect adjacent to player
                        let offset = 20;  // Fixed offset for punch positioning
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
                    // Left click: Start painting
                    is_painting = true;
                    let tile_x = x / 32;
                    let tile_y = y / 32;

                    // Place initial tile
                    if world_grid.set_tile(tile_x, tile_y, selected_tile) {
                        render_grid.update_tile_and_neighbors(&world_grid, tile_x, tile_y);
                        last_painted_tile = Some((tile_x, tile_y));
                    }
                }
                Event::MouseButtonUp { mouse_btn: sdl2::mouse::MouseButton::Left, .. } => {
                    // Release left mouse: Stop painting
                    is_painting = false;
                    last_painted_tile = None;
                }
                Event::MouseMotion { x, y, .. } => {
                    // Handle painting while dragging
                    if is_painting {
                        let tile_x = x / 32;
                        let tile_y = y / 32;

                        // Only paint if we've moved to a different tile
                        if last_painted_tile != Some((tile_x, tile_y)) {
                            if world_grid.set_tile(tile_x, tile_y, selected_tile) {
                                render_grid.update_tile_and_neighbors(&world_grid, tile_x, tile_y);
                                last_painted_tile = Some((tile_x, tile_y));
                            }
                        }
                    }
                }
                Event::MouseButtonDown { mouse_btn: sdl2::mouse::MouseButton::Right, x, y, .. } => {
                    // Spawn a new slime at mouse position
                    // Using factory method - clean and simple!
                    let slime_animation_controller = slime_config.create_controller(
                        &slime_texture,
                        &["slime_idle", "jump"],
                    )?;
                    // Center slime on click (32 * SPRITE_SCALE / 2 = offset)
                    let center_offset = (32 * SPRITE_SCALE / 2) as i32;
                    slimes.push(Slime::new(x - center_offset, y - center_offset, slime_animation_controller));
                }
                _ => {}
            }
        }

        // Only update game state when not in menu
        if game_state == GameState::Playing {
            // Update player
            let keyboard_state = event_pump.keyboard_state();
            player.update(&keyboard_state);
            // Bounds handled by collision system with static boundary objects

            // Update slimes
            for slime in &mut slimes {
                slime.update();
            }
        }

        // Update attack effects
        for effect in &mut attack_effects {
            effect.update();
        }

        // Remove finished attack effects
        attack_effects.retain(|effect| !effect.is_finished());

        // Attack hit detection
        // Check if active attack hits any slimes
        if let Some(ref attack) = active_attack {
            let attack_hitbox = attack.get_hitbox();

            for slime in &mut slimes {
                let slime_bounds = slime.get_bounds();

                // Check if attack hitbox intersects with slime
                if collision::aabb_intersect(&attack_hitbox, &slime_bounds) {
                    // Hit! Deal damage to slime
                    slime.take_damage(attack.damage as i32);
                }
            }

            // Clear attack after processing (attacks are one-frame)
            active_attack = None;
        }

        // Collision detection and response (for pushing, not damage)
        // Game loop pattern: Update → Collision → Render
        //
        // Rust Learning Note: Borrowing Challenge!
        // We can't borrow `player` and `slimes` mutably at the same time in a simple loop.
        // Solution: First detect collisions (immutable borrow), then resolve them (mutable).
        let colliding_slime_indices = check_collisions_with_collection(&player, &slimes);

        // Resolve collisions with push-apart and damage from contact
        // Strategy: Player is heavier, so slimes get pushed more (70/30 split)
        for slime_index in colliding_slime_indices {
            let player_bounds = player.get_bounds();
            let slime_bounds = slimes[slime_index].get_bounds();

            let (overlap_x, overlap_y) = calculate_overlap(&player_bounds, &slime_bounds);

            // Use the axis with smaller overlap for push-apart (minimum separation)
            if overlap_x.abs() < overlap_y.abs() {
                // Push apart on X axis
                player.apply_push(-overlap_x * 3 / 10, 0);
                slimes[slime_index].apply_push(overlap_x * 7 / 10, 0);
            } else {
                // Push apart on Y axis
                player.apply_push(0, -overlap_y * 3 / 10);
                slimes[slime_index].apply_push(0, overlap_y * 7 / 10);
            }

            // Contact damage: slime touches player (not while attacking)
            if !player.is_attacking {
                let damage = DamageEvent::physical(1.0, DamageSource::Enemy);
                player.take_damage(damage);
            }
        }

        // Remove dead slimes
        // Rust Learning: retain() is idiomatic for removing items from a Vec while iterating
        slimes.retain(|slime| slime.is_alive);

        // Static object collision (walls, obstacles)
        // One-way collision: only the player gets pushed, static objects never move
        let static_collisions = check_static_collisions(&player, &static_objects);

        for obj_index in static_collisions {
            let player_bounds = player.get_bounds();
            let obj_bounds = static_objects[obj_index].get_bounds();

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

        // Clear screen with black background
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        // Render background (old static background - can remove later)
        // canvas.copy(&background_texture, None, None)?;

        // Render tile world
        render_grid.render(&mut canvas, &grass_tile_texture)?;

        // Render static objects (simple gray rectangles for now)
        // Temporarily commented out to see tiles
        // canvas.set_draw_color(sdl2::pixels::Color::RGB(100, 100, 100));
        // for static_obj in &static_objects {
        //     let rect = sdl2::rect::Rect::new(
        //         static_obj.x,
        //         static_obj.y,
        //         static_obj.width,
        //         static_obj.height,
        //     );
        //     canvas.fill_rect(rect).map_err(|e| e.to_string())?;
        // }

        // Render player
        player.render(&mut canvas)?;

        // Render slimes
        for slime in &slimes {
            slime.render(&mut canvas)?;
        }

        // Render attack effects (on top of player/slimes to show attack range)
        for effect in &attack_effects {
            effect.render(&mut canvas, SPRITE_SCALE)?;
        }

        // Debug: Render collision boxes (toggle with 'B' key)
        if show_collision_boxes {
            canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 0, 0, 128));

            // Player collision box
            let player_bounds = player.get_bounds();
            canvas.draw_rect(player_bounds).map_err(|e| e.to_string())?;

            // Slime collision boxes
            for slime in &slimes {
                let slime_bounds = slime.get_bounds();
                canvas.draw_rect(slime_bounds).map_err(|e| e.to_string())?;
            }

            // Attack hitbox (if attacking)
            if let Some(ref attack) = active_attack {
                canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 255, 0, 128));
                let attack_hitbox = attack.get_hitbox();
                canvas.draw_rect(attack_hitbox).map_err(|e| e.to_string())?;
            }
        }

        // Debug: Render tile grid (toggle with 'G' key)
        if show_tile_grid {
            // Draw world grid lines (YELLOW)
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

            // Draw world tile type indicators (GREEN for grass, BROWN for dirt)
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

        // Debug info (optional)
        if false {
            // Set to true to see debug info
            println!(
                "Player: pos=({}, {}), vel=({}, {}), state={:?}",
                player.position().0,
                player.position().1,
                player.velocity().0,
                player.velocity().1,
                player.current_animation_state()
            );
        }

        // Render exit menu if active
        if game_state == GameState::ExitMenu {
            render_exit_menu(&mut canvas, exit_menu_selection)?;
        }

        canvas.present();

        // Cap framerate to ~60 FPS
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
