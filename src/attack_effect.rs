use crate::animation::{AnimationController, Direction};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// AttackEffect represents a visual effect that shows the range/hitbox of an attack.
///
/// Game Dev Pattern: Visual Effects (VFX)
/// This is separate from the player's character animation. When the player attacks,
/// we spawn one of these effects positioned in front of them to show the attack's range.
/// Once the animation finishes playing, the effect is removed.
pub struct AttackEffect<'a> {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    #[allow(dead_code)] // Keeping for future directional effect support
    pub direction: Direction,
    animation_controller: AnimationController<'a>,
}

impl<'a> AttackEffect<'a> {
    /// Creates a new attack effect at the specified position.
    ///
    /// # Parameters
    /// - `x`, `y`: Position to render the effect
    /// - `width`, `height`: Size of each frame (32x32 for your punch sprite)
    /// - `direction`: Which direction the attack is facing (for future directional effects)
    /// - `animation_controller`: Controller loaded with the punch animation
    pub fn new(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        direction: Direction,
        mut animation_controller: AnimationController<'a>,
    ) -> Self {
        // IMPORTANT: Set the initial state to "punch" so the animation plays!
        // AnimationController starts with an empty state by default.
        animation_controller.set_state("punch".to_string());

        AttackEffect {
            x,
            y,
            width,
            height,
            direction,
            animation_controller,
        }
    }

    /// Updates the effect's animation.
    ///
    /// Call this once per frame in the game loop.
    pub fn update(&mut self) {
        self.animation_controller.update();
    }

    /// Renders the attack effect to the screen.
    ///
    /// # Parameters
    /// - `canvas`: The SDL2 canvas to draw on
    /// - `scale`: Rendering scale (usually 3 to match player scale)
    pub fn render(&self, canvas: &mut Canvas<Window>, scale: u32) -> Result<(), String> {
        let scaled_width = self.width * scale;
        let scaled_height = self.height * scale;
        let dest_rect = Rect::new(self.x, self.y, scaled_width, scaled_height);

        if let Some(sprite_sheet) = self.animation_controller.get_current_sprite_sheet() {
            // Rotate sprite to match direction
            sprite_sheet.render_rotated(canvas, dest_rect, self.direction)
        } else {
            // Fallback: render a semi-transparent red box to show where the effect would be
            canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 0, 0, 128));
            canvas.fill_rect(dest_rect).map_err(|e| e.to_string())
        }
    }

    /// Returns true if the animation has finished playing.
    ///
    /// Use this to remove the effect from the game world after it's done.
    pub fn is_finished(&self) -> bool {
        self.animation_controller.is_animation_finished()
    }
}
