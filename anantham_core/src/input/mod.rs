use bevy_ecs::prelude::Resource;
use std::collections::HashSet;
use winit::keyboard::KeyCode;

#[derive(Resource, Default)]
pub struct Input {
    pressed_keys: HashSet<KeyCode>,
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
}
