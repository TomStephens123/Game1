use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;

mod animation;
mod player;
mod slime;
mod sprite;

use animation::{AnimationConfig, AnimationController, AnimationState};
use player::Player;
use slime::Slime;
use sprite::SpriteSheet;


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

fn setup_player_animations<'a>(
    character_texture: &'a sdl2::render::Texture<'a>,
    config: &AnimationConfig,
) -> Result<AnimationController<'a>, String> {
    let mut controller = AnimationController::new();

    // Create IDLE animation using Character-Base texture
    let idle_frames = config.create_frames(&AnimationState::Idle);
    let mut idle_sprite_sheet = SpriteSheet::new(character_texture, idle_frames);
    idle_sprite_sheet.set_loop(config.should_loop(&AnimationState::Idle));
    idle_sprite_sheet.set_animation_mode(config.get_animation_mode(&AnimationState::Idle));
    controller.add_animation(AnimationState::Idle, idle_sprite_sheet);

    // Create RUNNING animation using Character-Base texture
    let run_frames = config.create_frames(&AnimationState::Running);
    let mut run_sprite_sheet = SpriteSheet::new(character_texture, run_frames);
    run_sprite_sheet.set_loop(config.should_loop(&AnimationState::Running));
    run_sprite_sheet.set_animation_mode(config.get_animation_mode(&AnimationState::Running));
    controller.add_animation(AnimationState::Running, run_sprite_sheet);

    // Create ATTACK animation using Character-Base texture
    let attack_frames = config.create_frames(&AnimationState::Attack);
    let mut attack_sprite_sheet = SpriteSheet::new(character_texture, attack_frames);
    attack_sprite_sheet.set_loop(config.should_loop(&AnimationState::Attack));
    attack_sprite_sheet.set_animation_mode(config.get_animation_mode(&AnimationState::Attack));
    controller.add_animation(AnimationState::Attack, attack_sprite_sheet);

    Ok(controller)
}

fn create_slime_animation_controller<'a>(
    slime_texture: &'a sdl2::render::Texture<'a>,
    config: &AnimationConfig,
) -> Result<AnimationController<'a>, String> {
    let mut controller = AnimationController::new();

    // Create SlimeIdle animation
    let idle_frames = config.create_frames(&AnimationState::SlimeIdle);
    let mut idle_sprite_sheet = SpriteSheet::new(slime_texture, idle_frames);
    idle_sprite_sheet.set_loop(config.should_loop(&AnimationState::SlimeIdle));
    idle_sprite_sheet.set_animation_mode(config.get_animation_mode(&AnimationState::SlimeIdle));
    controller.add_animation(AnimationState::SlimeIdle, idle_sprite_sheet);

    // Create Jump animation
    let jump_frames = config.create_frames(&AnimationState::Jump);
    let mut jump_sprite_sheet = SpriteSheet::new(slime_texture, jump_frames);
    jump_sprite_sheet.set_loop(config.should_loop(&AnimationState::Jump));
    jump_sprite_sheet.set_animation_mode(config.get_animation_mode(&AnimationState::Jump));
    controller.add_animation(AnimationState::Jump, jump_sprite_sheet);

    Ok(controller)
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG)?;

    let window = video_subsystem
        .window("Game 1 - 8-Directional Character Animation", 1028, 1028)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump()?;

    // Load animation configurations
    let player_config = AnimationConfig::load_from_file("assets/config/player_animations.json")
        .map_err(|e| format!("Failed to load player animation config: {}", e))?;
    let slime_config = AnimationConfig::load_from_file("assets/config/slime_animations.json")
        .map_err(|e| format!("Failed to load slime animation config: {}", e))?;

    // Load sprite textures
    let character_texture = load_character_texture(&texture_creator)?;
    let slime_texture = load_slime_texture(&texture_creator)?;
    let background_texture = load_background_texture(&texture_creator)?;

    // Setup player with animations (RUNNING and ATTACK)
    let animation_controller = setup_player_animations(&character_texture, &player_config)?;
    let mut player = Player::new(300, 200, 32, 32, 3); // Adjusted position for larger sprites
    player.set_animation_controller(animation_controller);

    // Vector to store slimes spawned by mouse clicks
    let mut slimes: Vec<Slime> = Vec::new();

    println!("Controls:");
    println!("WASD - Move player");
    println!("M Key - Attack");
    println!("Mouse Click - Spawn slime");
    println!("ESC - Exit");
    println!("\nDemo Features:");
    println!("- 8-directional character movement and animation");
    println!("- 2-frame idle animation (300ms per frame)");
    println!("- 2-frame walking animation (150ms per frame)");
    println!("- 3-frame fist attack animation (100ms per frame, non-looping)");
    println!("- Directional sprites for all 8 directions (S, SE, E, NE, N, NW, W, SW)");
    println!("- Slime enemies with idle/jump behavior cycle");
    println!("- 3-frame slime idle animation (ping-pong playback)");
    println!("- 3-frame slime jump animation (ping-pong playback)");
    println!("- Slimes jump once every 2 seconds");
    println!("- Click to spawn slimes at cursor position");
    println!("- Tactical combat: No horizontal movement during attacks");
    println!("- Vertical movement allowed during attacks for positioning");

    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::M),
                    ..
                } => {
                    player.start_attack();
                }
                Event::MouseButtonDown { x, y, .. } => {
                    let slime_animation_controller = create_slime_animation_controller(&slime_texture, &slime_config)?;
                    slimes.push(Slime::new(x - 48, y - 48, slime_animation_controller)); // Center larger slime on click (32*3/2 = 48)
                }
                _ => {}
            }
        }

        // Update player
        let keyboard_state = event_pump.keyboard_state();
        player.update(&keyboard_state);
        player.keep_in_bounds(1028, 1028);

        // Update slimes
        for slime in &mut slimes {
            slime.update();
        }

        // Clear screen
        canvas.clear();

        // Render background
        canvas.copy(&background_texture, None, None)?;

        // Render player
        player.render(&mut canvas)?;

        // Render slimes
        for slime in &slimes {
            slime.render(&mut canvas)?;
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

        canvas.present();

        // Cap framerate to ~60 FPS
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
