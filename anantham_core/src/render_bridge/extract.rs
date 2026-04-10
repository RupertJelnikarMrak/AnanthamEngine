use crate::render_bridge::components::ExtractedView;
use crate::spatial::{Camera, Transform};
use bevy_ecs::prelude::*;
use glam::Mat4;

pub fn extract_camera_system(main_world: &mut World, render_world: &mut World) {
    let mut query = main_world.query::<(&Camera, &Transform)>();

    if let Some((camera, transform)) = query.iter(main_world).next() {
        let view = transform.compute_matrix().inverse();

        let aspect_ratio = 16.0 / 9.0;
        let proj = Mat4::perspective_rh(camera.fov, aspect_ratio, camera.near, camera.far);

        let mut vulkan_proj = proj;
        vulkan_proj.y_axis.y *= -1.0;

        let view_projection = vulkan_proj * view;

        render_world.insert_resource(ExtractedView { view_projection });
    }
}
