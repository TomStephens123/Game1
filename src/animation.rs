use crate::sprite::{Frame, SpriteSheet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimationState {
    Idle,
    Running,
    Attack,
}

impl Default for AnimationState {
    fn default() -> Self {
        AnimationState::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    pub frame_width: u32,
    pub frame_height: u32,
    pub animations: HashMap<AnimationState, AnimationData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationData {
    pub frames: Vec<FrameData>,
    pub loop_animation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameData {
    pub x: i32,
    pub y: i32,
    pub duration_ms: u64,
}

impl FrameData {
    pub fn to_frame(&self, width: u32, height: u32) -> Frame {
        Frame::new(self.x, self.y, width, height, self.duration_ms)
    }
}

pub struct AnimationController<'a> {
    current_state: AnimationState,
    previous_state: AnimationState,
    sprite_sheets: HashMap<AnimationState, SpriteSheet<'a>>,
    state_changed: bool,
}

impl<'a> AnimationController<'a> {
    pub fn new() -> Self {
        AnimationController {
            current_state: AnimationState::default(),
            previous_state: AnimationState::default(),
            sprite_sheets: HashMap::new(),
            state_changed: false,
        }
    }

    pub fn add_animation(&mut self, state: AnimationState, sprite_sheet: SpriteSheet<'a>) {
        self.sprite_sheets.insert(state, sprite_sheet);
    }

    pub fn set_state(&mut self, new_state: AnimationState) {
        if new_state != self.current_state {
            self.previous_state = self.current_state.clone();
            self.current_state = new_state;
            self.state_changed = true;
        }
    }

    pub fn update(&mut self) {
        if self.state_changed {
            if let Some(sprite_sheet) = self.sprite_sheets.get_mut(&self.current_state) {
                sprite_sheet.reset();
                sprite_sheet.play();
            }
            self.state_changed = false;
        }

        if let Some(sprite_sheet) = self.sprite_sheets.get_mut(&self.current_state) {
            sprite_sheet.update();
        }
    }

    pub fn get_current_sprite_sheet(&self) -> Option<&SpriteSheet<'a>> {
        self.sprite_sheets.get(&self.current_state)
    }

    pub fn current_state(&self) -> &AnimationState {
        &self.current_state
    }

    pub fn is_animation_finished(&self) -> bool {
        if let Some(sprite_sheet) = self.sprite_sheets.get(&self.current_state) {
            sprite_sheet.is_finished()
        } else {
            false
        }
    }
}

impl AnimationConfig {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: AnimationConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn create_frames(&self, state: &AnimationState) -> Vec<Frame> {
        if let Some(animation_data) = self.animations.get(state) {
            animation_data
                .frames
                .iter()
                .map(|frame_data| frame_data.to_frame(self.frame_width, self.frame_height))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn should_loop(&self, state: &AnimationState) -> bool {
        self.animations
            .get(state)
            .map(|data| data.loop_animation)
            .unwrap_or(true)
    }
}

pub fn determine_animation_state(
    velocity_x: i32,
    velocity_y: i32,
    _speed_threshold: i32,
) -> AnimationState {
    let total_velocity = (velocity_x.abs() + velocity_y.abs()) as i32;

    if total_velocity == 0 {
        AnimationState::Idle
    } else {
        AnimationState::Running
    }
}
