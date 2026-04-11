pub mod context;
pub mod core;
pub mod pipeline;
pub mod resource;

use anantham_core::{
    App, AppWindow, Plugin,
    render_bridge::components::{ExtractedMeshes, ExtractedView},
};
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

fn draw_frame_system(
    vulkan_context: Option<ResMut<VulkanContext>>,
    extracted_view: Option<Res<ExtractedView>>,
    extracted_meshes: Option<Res<ExtractedMeshes>>,
) {
    if let Some(mut context) = vulkan_context {
        context
            .draw_frame(extracted_view.as_deref(), extracted_meshes.as_deref())
            .expect("Failed to draw frame");
    }
}
