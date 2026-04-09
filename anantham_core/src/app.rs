use bevy_ecs::prelude::*;

pub trait Plugin {
    fn build(&self, app: &mut App);
}

pub struct App {
    pub main_world: World,
    pub render_world: World,
    pub main_schedule: Schedule,
    pub extract_schedule: Schedule,
    pub render_schedule: Schedule,
}

impl Default for App {
    fn default() -> Self {
        Self {
            main_world: World::new(),
            render_world: World::new(),
            main_schedule: Schedule::default(),
            extract_schedule: Schedule::default(),
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

    /// The core execution pipeline, called every frame
    pub fn update(&mut self) {
        // 1. Run Game Logic
        self.main_schedule.run(&mut self.main_world);

        // 2. Extract Phase (Note: bevy_ecs doesn't natively run schedules
        // across two worlds out-of-the-box, so we will handle the bridge
        // via dedicated extraction systems later. For now, we run it on the main world).
        self.extract_schedule.run(&mut self.main_world);

        // 3. Draw Phase
        self.render_schedule.run(&mut self.render_world);
    }
}
