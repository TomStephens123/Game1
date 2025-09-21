use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::image::LoadTexture;

mod sprite;
mod animation;
mod player;

use sprite::SpriteSheet;
use animation::{AnimationController, AnimationConfig, AnimationState};
use player::Player;

// TODO: Multi-texture system for different animation states
// Currently only loads IDLE sprite - need to expand this to support:
// - Loading multiple textures (IDLE.png, RUN.png, HURT.png, ATTACK 1.png)
// - Mapping animation states to appropriate textures
// - Resource management for efficient texture caching
// - Dynamic switching between textures based on animation state

fn load_sprite_textures(texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> Result<(sdl2::render::Texture<'_>, sdl2::render::Texture<'_>, sdl2::render::Texture<'_>), String> {
    let idle_texture = texture_creator
        .load_texture("assets/sprites/player/IDLE.png")
        .map_err(|e| format!("Failed to load IDLE.png: {}", e))?;

    let run_texture = texture_creator
        .load_texture("assets/sprites/player/RUN.png")
        .map_err(|e| format!("Failed to load RUN.png: {}", e))?;

    let attack_texture = texture_creator
        .load_texture("assets/sprites/player/ATTACK 1.png")
        .map_err(|e| format!("Failed to load ATTACK 1.png: {}", e))?;

    Ok((idle_texture, run_texture, attack_texture))
}

fn setup_player_animations<'a>(
    idle_texture: &'a sdl2::render::Texture<'a>,
    run_texture: &'a sdl2::render::Texture<'a>,
    attack_texture: &'a sdl2::render::Texture<'a>,
    config: &AnimationConfig,
) -> Result<AnimationController<'a>, String> {
    let mut controller = AnimationController::new();

    // Create IDLE animation using IDLE texture
    let idle_frames = config.create_frames(&AnimationState::Idle);
    let mut idle_sprite_sheet = SpriteSheet::new(idle_texture, idle_frames);
    idle_sprite_sheet.set_loop(config.should_loop(&AnimationState::Idle));
    controller.add_animation(AnimationState::Idle, idle_sprite_sheet);

    // Create RUNNING animation using RUN texture
    let run_frames = config.create_frames(&AnimationState::Running);
    let mut run_sprite_sheet = SpriteSheet::new(run_texture, run_frames);
    run_sprite_sheet.set_loop(config.should_loop(&AnimationState::Running));
    controller.add_animation(AnimationState::Running, run_sprite_sheet);

    // Create ATTACK animation using ATTACK texture
    let attack_frames = config.create_frames(&AnimationState::Attack);
    let mut attack_sprite_sheet = SpriteSheet::new(attack_texture, attack_frames);
    attack_sprite_sheet.set_loop(config.should_loop(&AnimationState::Attack));
    controller.add_animation(AnimationState::Attack, attack_sprite_sheet);

    Ok(controller)
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG)?;

    let window = video_subsystem
        .window("Game 1 - Real Sprite Demo (IDLE + RUN Animations)", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump()?;

    // Load animation configuration
    let config = AnimationConfig::load_from_file("assets/config/player_animations.json")
        .map_err(|e| format!("Failed to load animation config: {}", e))?;

    // Load all sprite textures
    let (idle_texture, run_texture, attack_texture) = load_sprite_textures(&texture_creator)?;

    // Setup player with animations (IDLE, RUN, and ATTACK)
    let animation_controller = setup_player_animations(&idle_texture, &run_texture, &attack_texture, &config)?;
    let mut player = Player::new(375, 275, 96, 96, 3);
    player.set_animation_controller(animation_controller);

    println!("Controls:");
    println!("WASD - Move player");
    println!("Mouse Click - Attack");
    println!("ESC - Exit");
    println!("\nDemo Features:");
    println!("- Real 16-bit sprite animations (IDLE + RUN + ATTACK)");
    println!("- 10-frame IDLE animation (100ms per frame)");
    println!("- 16-frame RUN animation (40ms per frame)");
    println!("- 7-frame ATTACK animation (60ms per frame, non-looping)");
    println!("- Tactical combat: No horizontal movement during attacks");
    println!("- Vertical movement allowed during attacks for positioning");
    println!("- Mouse click attacks with priority over movement");
    println!("- Sprite flipping for direction changes");

    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown { .. } => {
                    player.start_attack();
                }
                _ => {}
            }
        }

        // Update player
        let keyboard_state = event_pump.keyboard_state();
        player.update(&keyboard_state);
        player.keep_in_bounds(800, 600);

        // Clear screen with blue background
        canvas.set_draw_color(sdl2::pixels::Color::RGB(64, 128, 255));
        canvas.clear();

        // Render player
        player.render(&mut canvas)?;

        // Debug info (optional)
        if false { // Set to true to see debug info
            println!("Player: pos=({}, {}), vel=({}, {}), state={:?}",
                player.position().0, player.position().1,
                player.velocity().0, player.velocity().1,
                player.current_animation_state());
        }

        canvas.present();

        // Cap framerate to ~60 FPS
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
