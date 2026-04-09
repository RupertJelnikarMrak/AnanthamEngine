use crate::app::App;
use bevy_ecs::prelude::Resource;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[derive(Resource, Clone)]
pub struct AppWindow(pub Arc<Window>);

pub struct EngineRunner {
    pub app: App,
    window: Option<Arc<Window>>,
}

impl EngineRunner {
    pub fn new(app: App) -> Self {
        Self { app, window: None }
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self)?;
        Ok(())
    }
}

impl ApplicationHandler for EngineRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attributes = Window::default_attributes()
                .with_title("Anantham Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

            let window = Arc::new(event_loop.create_window(attributes).unwrap());

            // Insert the window into the Render World so the Vulkan plugin can find it
            self.app
                .render_world
                .insert_resource(AppWindow(window.clone()));
            self.window = Some(window);

            tracing::info!("Window created and injected into Render World");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Shutting down engine...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Execute the full game and render loop
                self.app.update();

                // Request the next frame immediately
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}
