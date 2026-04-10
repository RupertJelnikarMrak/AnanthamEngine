use crate::input::Input;
use crate::{ScreenResolution, app::App};
use bevy_ecs::prelude::Resource;
use std::sync::Arc;
use winit::event::DeviceEvent;
use winit::keyboard::KeyCode;
use winit::window::CursorGrabMode;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
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

            let physical_size = window.inner_size();
            self.app.main_world.insert_resource(ScreenResolution {
                width: physical_size.width,
                height: physical_size.height,
            });

            self.app
                .render_world
                .insert_resource(AppWindow(window.clone()));
            self.window = Some(window);

            tracing::info!("Window created and injected into Render World");
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event
            && let Some(mut input) = self.app.main_world.get_resource_mut::<Input>()
        {
            input.add_mouse_delta(delta.0 as f32, delta.1 as f32);
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

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                if let Some(window) = &self.window {
                    let _ = window
                        .set_cursor_grab(CursorGrabMode::Confined)
                        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
                    window.set_cursor_visible(false);
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state,
                        ..
                    },
                ..
            } => {
                if keycode == KeyCode::Escape
                    && state == ElementState::Pressed
                    && let Some(window) = &self.window
                {
                    let _ = window.set_cursor_grab(CursorGrabMode::None);
                    window.set_cursor_visible(true);
                }
                if let Some(mut input) = self.app.main_world.get_resource_mut::<Input>() {
                    match state {
                        ElementState::Pressed => input.press(keycode),
                        ElementState::Released => input.release(keycode),
                    }
                }
            }

            WindowEvent::Resized(physical_size) => {
                if let Some(mut res) = self.app.main_world.get_resource_mut::<ScreenResolution>() {
                    res.width = physical_size.width;
                    res.height = physical_size.height;
                }
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
