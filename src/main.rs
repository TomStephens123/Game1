use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Game 1", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    // Player position and movement speed
    let mut player_x = 375; // Center horizontally (800/2 - 25)
    let mut player_y = 275; // Center vertically (600/2 - 25)
    let player_speed = 2;
    let player_size = 50u32;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Handle WASD movement
        let keyboard_state = event_pump.keyboard_state();

        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::W) {
            player_y -= player_speed;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::S) {
            player_y += player_speed;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::A) {
            player_x -= player_speed;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::D) {
            player_x += player_speed;
        }

        // Keep player within window bounds
        if player_x < 0 { player_x = 0; }
        if player_y < 0 { player_y = 0; }
        if player_x > 800 - player_size as i32 { player_x = 800 - player_size as i32; }
        if player_y > 600 - player_size as i32 { player_y = 600 - player_size as i32; }

        // Clear screen with blue background
        canvas.set_draw_color(sdl2::pixels::Color::RGB(64, 128, 255));
        canvas.clear();

        // Draw red square (player)
        canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 0, 0));
        let player_rect = sdl2::rect::Rect::new(player_x, player_y, player_size, player_size);
        canvas.fill_rect(player_rect)?;

        canvas.present();
    }

    Ok(())
}
