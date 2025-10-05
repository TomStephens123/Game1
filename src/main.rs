use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;

mod animation;
mod attack_effect;
mod collision;
mod player;
mod slime;
mod sprite;

use animation::AnimationConfig;
use attack_effect::AttackEffect;
use collision::{
    calculate_overlap, check_collisions_with_collection, check_static_collisions, Collidable,
    StaticCollidable, StaticObject,
};
use player::Player;
use slime::Slime;


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
    let punch_config = AnimationConfig::load_from_file("assets/config/punch_effect.json")
        .map_err(|e| format!("Failed to load punch effect config: {}", e))?;

    // Validation: Print available animations (helpful during development)
    println!("\n=== Animation System Loaded ===");
    println!("Player animations: {:?}", player_config.available_states());
    println!("Slime animations: {:?}", slime_config.available_states());
    println!("Punch effect animations: {:?}", punch_config.available_states());

    // Load sprite textures
    let character_texture = load_character_texture(&texture_creator)?;
    let slime_texture = load_slime_texture(&texture_creator)?;
    let background_texture = load_background_texture(&texture_creator)?;
    let punch_texture = load_punch_texture(&texture_creator)?;

    // Setup player with animations using new factory function
    // Game Dev Pattern: This single line replaces 27 lines of boilerplate!
    let animation_controller = player_config.create_controller(
        &character_texture,
        &["idle", "running", "attack"],
    )?;
    let mut player = Player::new(300, 200, 32, 32, 3);
    player.set_animation_controller(animation_controller);

    // Hitbox is already tuned to sprite artwork (16x16 centered)
    // Use player.set_hitbox() to adjust if needed, or press 'B' to visualize

    // Vector to store slimes spawned by mouse clicks
    let mut slimes: Vec<Slime> = Vec::new();

    // Vector to store active attack effects (punch visuals)
    let mut attack_effects: Vec<AttackEffect> = Vec::new();

    // Debug toggle for collision visualization
    let mut show_collision_boxes = false;

    // Create static world objects (obstacles)
    // These are immovable objects the player cannot pass through
    let static_objects = vec![
        // Top border walls
        StaticObject::new(0, 0, 1028, 32),
        // Left border wall
        StaticObject::new(0, 0, 32, 1028),
        // Right border wall
        StaticObject::new(1028 - 32, 0, 32, 1028),
        // Bottom border wall
        StaticObject::new(0, 1028 - 32, 1028, 32),
        // Central obstacle (rock/building)
        StaticObject::new(450, 400, 128, 128),
    ];

    println!("Controls:");
    println!("WASD - Move player");
    println!("M Key - Attack");
    println!("B Key - Toggle collision debug boxes");
    println!("Mouse Click - Spawn slime");
    println!("ESC - Exit");
    println!("\nDemo Features:");
    println!("- Click to spawn slimes at cursor position");
    println!("- Press M to attack");
    println!("\n=== NEW: Collision System ===");
    println!("- Push-apart physics prevents overlap");
    println!("- Touching slimes without attacking damages player (10 HP total)");
    println!("- 1 second invulnerability after taking damage");
    println!("- Static world objects (gray rectangles are solid walls)");
    println!("- Border walls and central obstacle block movement");

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

                    // Spawn punch effect in front of player based on their direction
                    // Calculate offset based on player's facing direction
                    let (offset_x, offset_y) = match player.direction {
                        crate::animation::Direction::North => (0, -32),
                        crate::animation::Direction::NorthEast => (32, -32),
                        crate::animation::Direction::East => (32, 0),
                        crate::animation::Direction::SouthEast => (32, 32),
                        crate::animation::Direction::South => (0, 32),
                        crate::animation::Direction::SouthWest => (-32, 32),
                        crate::animation::Direction::West => (-32, 0),
                        crate::animation::Direction::NorthWest => (-32, -32),
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
                Event::KeyDown {
                    keycode: Some(Keycode::B),
                    ..
                } => {
                    show_collision_boxes = !show_collision_boxes;
                    println!("Collision boxes: {}", if show_collision_boxes { "ON" } else { "OFF" });
                }
                Event::MouseButtonDown { x, y, .. } => {
                    // Spawn a new slime at mouse position
                    // Using factory method - clean and simple!
                    let slime_animation_controller = slime_config.create_controller(
                        &slime_texture,
                        &["slime_idle", "jump"],
                    )?;
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

        // Update attack effects
        for effect in &mut attack_effects {
            effect.update();
        }

        // Remove finished attack effects
        attack_effects.retain(|effect| !effect.is_finished());

        // Collision detection and response
        // Game loop pattern: Update → Collision → Render
        //
        // Rust Learning Note: Borrowing Challenge!
        // We can't borrow `player` and `slimes` mutably at the same time in a simple loop.
        // Solution: First detect collisions (immutable borrow), then resolve them (mutable).
        let colliding_slime_indices = check_collisions_with_collection(&player, &slimes);

        // Resolve collisions with push-apart and damage
        // Strategy: Player is heavier, so slimes get pushed more (70/30 split)
        for slime_index in colliding_slime_indices {
            let player_bounds = player.get_bounds();
            let slime_bounds = slimes[slime_index].get_bounds();

            let (overlap_x, overlap_y) = calculate_overlap(&player_bounds, &slime_bounds);

            // Use the axis with smaller overlap for push-apart (minimum separation)
            if overlap_x.abs() < overlap_y.abs() {
                // Push apart on X axis
                // Negative overlap means player is to the right, so push player right, slime left
                player.apply_push(-overlap_x * 3 / 10, 0); // Player gets 30% of push
                slimes[slime_index].apply_push(overlap_x * 7 / 10, 0); // Slime gets 70% of push
            } else {
                // Push apart on Y axis
                player.apply_push(0, -overlap_y * 3 / 10);
                slimes[slime_index].apply_push(0, overlap_y * 7 / 10);
            }

            // Apply damage
            // If player is attacking, damage the slime. Otherwise, slime damages player.
            if player.is_attacking {
                slimes[slime_index].take_damage(1);
            } else {
                player.take_damage(1);
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

        // Clear screen
        canvas.clear();

        // Render background
        canvas.copy(&background_texture, None, None)?;

        // Render static objects (simple gray rectangles for now)
        canvas.set_draw_color(sdl2::pixels::Color::RGB(100, 100, 100));
        for static_obj in &static_objects {
            let rect = sdl2::rect::Rect::new(
                static_obj.x,
                static_obj.y,
                static_obj.width,
                static_obj.height,
            );
            canvas.fill_rect(rect).map_err(|e| e.to_string())?;
        }

        // Render player
        player.render(&mut canvas)?;

        // Render slimes
        for slime in &slimes {
            slime.render(&mut canvas)?;
        }

        // Render attack effects (on top of player/slimes to show attack range)
        for effect in &attack_effects {
            effect.render(&mut canvas, 3)?; // Scale 3x to match player scale
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
