use crate::animation::AnimationController;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
enum SlimeBehavior {
    Idle,
    Jumping,
}

pub struct Slime<'a> {
    pub x: i32,
    pub y: i32,
    pub base_y: i32, // Original Y position for jumping reference
    pub width: u32,
    pub height: u32,
    animation_controller: AnimationController<'a>,
    behavior: SlimeBehavior,
    behavior_timer: Instant,
    jump_height: i32,
    jump_duration: f32, // Duration of jump animation in seconds
}

impl<'a> Slime<'a> {
    pub fn new(x: i32, y: i32, animation_controller: AnimationController<'a>) -> Self {
        Slime {
            x,
            y,
            base_y: y,
            width: 32,
            height: 32,
            animation_controller,
            behavior: SlimeBehavior::Idle,
            behavior_timer: Instant::now(),
            jump_height: 20, // How high the slime bounces
            jump_duration: 0.5, // Jump lasts 0.5 seconds total (2x faster)
        }
    }

    pub fn update(&mut self) {
        let elapsed_time = self.behavior_timer.elapsed().as_secs_f32();

        // Game Dev Pattern: Simple AI State Machine
        // The slime alternates between idle and jumping based on timers
        match self.behavior {
            SlimeBehavior::Idle => {
                // Idle for 2 seconds, then switch to jumping
                if elapsed_time >= 2.0 {
                    self.behavior = SlimeBehavior::Jumping;
                    self.behavior_timer = Instant::now();
                    self.animation_controller.set_state("jump".to_string());
                } else {
                    // Make sure we're in idle animation
                    if self.animation_controller.current_state() != "slime_idle" {
                        self.animation_controller.set_state("slime_idle".to_string());
                    }
                }
                // Stay at base position when idle
                self.y = self.base_y;
            }
            SlimeBehavior::Jumping => {
                // Jump for jump_duration, then return to idle
                if elapsed_time >= self.jump_duration {
                    self.behavior = SlimeBehavior::Idle;
                    self.behavior_timer = Instant::now();
                    self.animation_controller.set_state("slime_idle".to_string());
                    self.y = self.base_y; // Return to ground
                } else {
                    // Calculate jump position using sine wave
                    // Game Dev Math: sin() gives smooth bounce motion (0 -> 1 -> 0)
                    let jump_progress = (elapsed_time * std::f32::consts::PI / self.jump_duration).sin();
                    let jump_offset = (jump_progress * self.jump_height as f32) as i32;
                    self.y = self.base_y - jump_offset;
                }
            }
        }

        // Always update animation controller
        self.animation_controller.update();
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let scale = 3; // 3x zoom scale
        let scaled_width = self.width * scale;
        let scaled_height = self.height * scale;
        let dest_rect = Rect::new(self.x, self.y, scaled_width, scaled_height);

        if let Some(sprite_sheet) = self.animation_controller.get_current_sprite_sheet() {
            sprite_sheet.render_flipped(canvas, dest_rect, false)
        } else {
            // Fallback red square if no sprite sheet
            canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 0, 0));
            canvas.fill_rect(dest_rect).map_err(|e| e.to_string())
        }
    }
}