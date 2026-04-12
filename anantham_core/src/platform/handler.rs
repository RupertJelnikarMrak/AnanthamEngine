use crate::platform::window::{AppWindow, MouseMotion, ScreenResolution};
use crate::prelude::*;
use crate::render_bridge::{ExtractSchedule, RenderSchedule};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

struct AnanthamHandler {
    app: App,
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for AnanthamHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attributes = Window::default_attributes().with_title("Anantham Engine");

            let window = Arc::new(event_loop.create_window(attributes).unwrap());

            self.app.insert_resource(AppWindow(window.clone()));
            self.window = Some(window);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId, // Note: Capital 'I' for winit 0.30+
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            // In Bevy 0.18.1, buffered events are Messages.
            // We fetch the Messages queue resource and push directly into it.
            if let Some(mut mouse_messages) = self
                .app
                .world_mut()
                .get_resource_mut::<Messages<MouseMotion>>()
            {
                mouse_messages.write(MouseMotion {
                    delta: glam::Vec2::new(delta.0 as f32, delta.1 as f32),
                });
            }
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
                // 1. Handle Hardcoded Window Shortcuts (Escape to un-grab mouse)
                if keycode == KeyCode::Escape
                    && state == ElementState::Pressed
                    && let Some(window) = &self.window
                {
                    let _ = window.set_cursor_grab(CursorGrabMode::None);
                    window.set_cursor_visible(true);
                }

                if keycode == KeyCode::F11
                    && state == ElementState::Pressed
                    && let Some(window) = &self.window
                {
                    let fullscreen = if window.fullscreen().is_some() {
                        None
                    } else {
                        Some(Fullscreen::Borderless(None))
                    };
                    window.set_fullscreen(fullscreen);
                }

                if let Some(mut input) = self
                    .app
                    .world_mut()
                    .get_resource_mut::<ButtonInput<KeyCode>>()
                {
                    match state {
                        ElementState::Pressed => input.press(keycode),
                        ElementState::Released => input.release(keycode),
                    }
                }
            }

            WindowEvent::Resized(physical_size) => {
                if let Some(mut res) = self.app.world_mut().get_resource_mut::<ScreenResolution>() {
                    res.width = physical_size.width;
                    res.height = physical_size.height;
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }

    // Triggered every frame when the OS event queue is empty
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            // Run Main ECS Logic
            self.app.update();

            // Extract changed data
            self.app.world_mut().run_schedule(ExtractSchedule);

            // Submit to Vulkan
            self.app.world_mut().run_schedule(RenderSchedule);
        }
    }
}

pub fn anantham_winit_runner(app: App) -> AppExit {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut runner = AnanthamHandler { app, window: None };

    event_loop.run_app(&mut runner).unwrap();

    AppExit::Success
}
