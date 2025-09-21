use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Frame {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub duration: Duration,
}

impl Frame {
    pub fn new(x: i32, y: i32, width: u32, height: u32, duration_ms: u64) -> Self {
        Frame {
            x,
            y,
            width,
            height,
            duration: Duration::from_millis(duration_ms),
        }
    }

    pub fn to_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

pub struct SpriteSheet<'a> {
    texture: &'a Texture<'a>,
    frames: Vec<Frame>,
    current_frame: usize,
    last_frame_time: Instant,
    is_playing: bool,
    loop_animation: bool,
}

impl<'a> SpriteSheet<'a> {
    pub fn new(texture: &'a Texture<'a>, frames: Vec<Frame>) -> Self {
        SpriteSheet {
            texture,
            frames,
            current_frame: 0,
            last_frame_time: Instant::now(),
            is_playing: true,
            loop_animation: true,
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
        self.last_frame_time = Instant::now();
    }

    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.last_frame_time = Instant::now();
    }

    pub fn set_loop(&mut self, should_loop: bool) {
        self.loop_animation = should_loop;
    }

    pub fn update(&mut self) {
        if !self.is_playing || self.frames.is_empty() {
            return;
        }

        let current_frame_duration = self.frames[self.current_frame].duration;

        if self.last_frame_time.elapsed() >= current_frame_duration {
            self.advance_frame();
            self.last_frame_time = Instant::now();
        }
    }

    fn advance_frame(&mut self) {
        if self.current_frame + 1 < self.frames.len() {
            self.current_frame += 1;
        } else if self.loop_animation {
            self.current_frame = 0;
        } else {
            self.is_playing = false;
        }
    }

    pub fn render_flipped(
        &self,
        canvas: &mut Canvas<Window>,
        dest_rect: Rect,
        flip_horizontal: bool,
    ) -> Result<(), String> {
        if self.frames.is_empty() {
            return Err("No frames to render".to_string());
        }

        let src_rect = self.frames[self.current_frame].to_rect();
        canvas
            .copy_ex(
                self.texture,
                Some(src_rect),
                Some(dest_rect),
                0.0,
                None,
                flip_horizontal,
                false,
            )
            .map_err(|e| e.to_string())
    }

    pub fn is_finished(&self) -> bool {
        !self.loop_animation && !self.is_playing && self.current_frame == self.frames.len() - 1
    }
}
