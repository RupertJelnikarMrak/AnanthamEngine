pub mod context;

use anantham_core::{App, AppWindow, Plugin};
use bevy_ecs::prelude::*;
use context::VulkanContext;

pub struct VoxelRenderPlugin;

impl Plugin for VoxelRenderPlugin {
    fn build(&self, app: &mut App) {
        app.render_schedule.add_systems((
            initialize_vulkan_system,
            draw_frame_system.after(initialize_vulkan_system),
        ));
    }
}

fn initialize_vulkan_system(
    mut commands: Commands,
    window: Option<Res<AppWindow>>,
    vulkan_context: Option<Res<VulkanContext>>,
) {
    if let Some(app_window) = window
        && vulkan_context.is_none()
    {
        tracing::info!("Initializing Vulkan Context...");
        let context =
            VulkanContext::new(&app_window.0).expect("Failed to initialize Vulkan Context");
        commands.insert_resource(context);
    }
}

fn draw_frame_system(vulkan_context: Option<ResMut<VulkanContext>>) {
    if let Some(mut context) = vulkan_context {
        context.draw_frame().expect("Failed to draw frame");
    }
}
