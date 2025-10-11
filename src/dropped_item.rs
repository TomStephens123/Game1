use crate::animation::AnimationController;
use crate::collision::{Collidable, CollisionLayer};
use crate::render::DepthSortable;
use crate::save::{Saveable, SaveData, SaveError};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use serde::{Serialize, Deserialize};
use std::time::Instant;

#[derive(Serialize, Deserialize)]
pub struct DroppedItemData {
    pub x: i32,
    pub y: i32,
    pub item_id: String,
    pub quantity: u32,
}

pub struct DroppedItem<'a> {
    pub x: i32,
    pub y: i32,
    pub item_id: String,
    pub quantity: u32,
    pub width: u32,
    pub height: u32,
    animation_controller: AnimationController<'a>,
    spawn_time: Instant,
    despawn_delay: f32,
    pub can_pickup: bool,
    pickup_cooldown: Instant,
    pickup_cooldown_duration: f32,
    pub pickup_radius: u32,
    render_y_offset: i32,
}

impl<'a> DroppedItem<'a> {
    pub fn new(
        x: i32,
        y: i32,
        item_id: String,
        quantity: u32,
        animation_controller: AnimationController<'a>,
    ) -> Self {
        DroppedItem {
            x,
            y,
            item_id,
            quantity,
            width: 32,
            height: 32,
            animation_controller,
            spawn_time: Instant::now(),
            despawn_delay: 300.0,
            can_pickup: false,
            pickup_cooldown: Instant::now(),
            pickup_cooldown_duration: 0.5,
            pickup_radius: 24,
            render_y_offset: 0,
        }
    }

    pub fn set_animation_controller(&mut self, controller: AnimationController<'a>) {
        self.animation_controller = controller;
    }

    pub fn update(&mut self) -> bool {
        self.animation_controller.update();
        let elapsed = self.spawn_time.elapsed().as_secs_f32();
        self.render_y_offset = (elapsed * 4.0).sin() as i32 * 3;
        if !self.can_pickup {
            if self.pickup_cooldown.elapsed().as_secs_f32() >= self.pickup_cooldown_duration {
                self.can_pickup = true;
            }
        }
        if elapsed >= self.despawn_delay {
            return true;
        }
        false
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let sprite_scale: u32 = if self.item_id == "slime_ball" {
            1
        } else {
            2
        };
        let scaled_width = self.width * sprite_scale;
        let scaled_height = self.height * sprite_scale;
        let render_x = self.x - (scaled_width / 2) as i32;
        let render_y = self.y - (scaled_height / 2) as i32 + self.render_y_offset;
        let dest_rect = Rect::new(render_x, render_y, scaled_width, scaled_height);
        if let Some(sprite_sheet) = self.animation_controller.get_current_sprite_sheet() {
            sprite_sheet.render_flipped(canvas, dest_rect, false)
        } else {
            canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 215, 0));
            canvas.fill_rect(dest_rect).map_err(|e| e.to_string())
        }
    }

    #[allow(dead_code)]
    pub fn time_until_despawn(&self) -> f32 {
        let age = self.spawn_time.elapsed().as_secs_f32();
        (self.despawn_delay - age).max(0.0)
    }

    #[allow(dead_code)]
    pub fn is_despawning_soon(&self) -> bool {
        self.time_until_despawn() < 10.0
    }

    #[allow(dead_code)]
    pub fn try_merge(&mut self, other: &DroppedItem, max_stack_size: u32) -> bool {
        if self.item_id != other.item_id {
            return false;
        }
        let total = self.quantity + other.quantity;
        self.quantity = total.min(max_stack_size);
        true
    }

    #[allow(dead_code)]
    pub fn from_item_stack(
        x: i32,
        y: i32,
        stack: &crate::item::ItemStack,
        animation_controller: AnimationController<'a>,
    ) -> Self {
        DroppedItem::new(x, y, stack.item_id.clone(), stack.quantity, animation_controller)
    }
}

impl DepthSortable for DroppedItem<'_> {
    fn get_depth_y(&self) -> i32 {
        self.y
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        DroppedItem::render(self, canvas)
    }
}

impl Collidable for DroppedItem<'_> {
    fn get_bounds(&self) -> Rect {
        let radius = self.pickup_radius as i32;
        Rect::new(
            self.x - radius,
            self.y - radius,
            self.pickup_radius * 2,
            self.pickup_radius * 2,
        )
    }

    fn get_collision_layer(&self) -> CollisionLayer {
        CollisionLayer::Item
    }
}

impl Saveable for DroppedItem<'_> {
    fn to_save_data(&self) -> Result<SaveData, SaveError> {
        let data = DroppedItemData {
            x: self.x,
            y: self.y,
            item_id: self.item_id.clone(),
            quantity: self.quantity,
        };
        Ok(SaveData {
            data_type: "dropped_item".to_string(),
            json_data: serde_json::to_string(&data)?,
        })
    }

    fn from_save_data(data: &SaveData) -> Result<Self, SaveError> where Self: Sized {
        if data.data_type != "dropped_item" {
            return Err(SaveError::CorruptedData(format!(
                "Expected dropped_item data, got {}",
                data.data_type
            )));
        }
        let item_data: DroppedItemData = serde_json::from_str(&data.json_data)?;
        // Animation controller is not saved, it will be recreated in main.rs
        let dummy_controller = AnimationController::new();
        let item = DroppedItem::new(
            item_data.x,
            item_data.y,
            item_data.item_id,
            item_data.quantity,
            dummy_controller,
        );
        Ok(item)
    }
}