use bevy_ecs::prelude::Resource;
use glam::Vec2;
use std::collections::HashSet;
use winit::keyboard::KeyCode;

#[derive(Resource, Default)]
pub struct Input {
    pressed_keys: HashSet<KeyCode>,
    pub mouse_delta: Vec2,
}

impl Input {
    pub fn press(&mut self, key: KeyCode) {
        self.pressed_keys.insert(key);
    }

    pub fn release(&mut self, key: KeyCode) {
        self.pressed_keys.remove(&key);
    }

    pub fn pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn add_mouse_delta(&mut self, x: f32, y: f32) {
        self.mouse_delta.x += x;
        self.mouse_delta.y += y;
    }

    pub fn take_mouse_delta(&mut self) -> Vec2 {
        let delta = self.mouse_delta;
        self.mouse_delta = Vec2::ZERO; // Reset after reading
        delta
    }
}
