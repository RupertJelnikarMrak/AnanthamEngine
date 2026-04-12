use crate::prelude::{Message, Resource};
use std::sync::Arc;
use winit::window::Window;

#[derive(Resource, Clone)]
pub struct AppWindow(pub Arc<Window>);

#[derive(Resource, Default, Clone, Copy)]
pub struct ScreenResolution {
    pub width: u32,
    pub height: u32,
}

impl ScreenResolution {
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            1.0
        } else {
            self.width as f32 / self.height as f32
        }
    }
}

#[derive(Message, Debug, Clone, Copy, Default)]
pub struct MouseMotion {
    pub delta: glam::Vec2,
}
