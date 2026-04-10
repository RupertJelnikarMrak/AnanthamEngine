pub mod log;
pub mod runner;

use bevy_ecs::prelude::*;

pub trait Plugin {
    fn build(&self, app: &mut App);
}

pub type ExtractSystem = Box<dyn FnMut(&mut World, &mut World)>;

pub struct App {
    pub main_world: World,
    pub render_world: World,
    pub main_schedule: Schedule,

    pub extract_systems: Vec<ExtractSystem>,

    pub render_schedule: Schedule,
}

impl Default for App {
    fn default() -> Self {
        let mut main_world = World::new();
        main_world.insert_resource(crate::input::Input::default());
        Self {
            main_world,
            render_world: World::new(),
            main_schedule: Schedule::default(),
            extract_systems: Vec::new(),
            render_schedule: Schedule::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_plugin<T: Plugin>(&mut self, plugin: T) -> &mut Self {
        plugin.build(self);
        self
    }

    /// Register a system that bridges the Main and Render worlds
    pub fn add_extract_system<F>(&mut self, system: F) -> &mut Self
    where
        F: FnMut(&mut World, &mut World) + 'static,
    {
        self.extract_systems.push(Box::new(system));
        self
    }

    /// The core execution pipeline, called every frame
    pub fn update(&mut self) {
        // 1. Run Game Logic
        self.main_schedule.run(&mut self.main_world);

        // 2. Synchronous Extract Phase
        for system in &mut self.extract_systems {
            system(&mut self.main_world, &mut self.render_world);
        }

        // 3. Draw Phase
        self.render_schedule.run(&mut self.render_world);
    }
}

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
