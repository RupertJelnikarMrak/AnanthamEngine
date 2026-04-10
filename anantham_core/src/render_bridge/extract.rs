use crate::ScreenResolution;
use crate::render_bridge::components::{ExtractedMesh, ExtractedMeshes, ExtractedView, Mesh};
use crate::spatial::{Camera, Transform};
use bevy_ecs::prelude::*;
use glam::Mat4;

pub fn extract_camera_system(main_world: &mut World, render_world: &mut World) {
    let resolution = main_world
        .get_resource::<ScreenResolution>()
        .copied()
        .unwrap_or_default();
    let aspect_ratio = resolution.aspect_ratio();

    let mut query = main_world.query::<(&Camera, &Transform)>();
    if let Some((camera, transform)) = query.iter(main_world).next() {
        let view = transform.compute_matrix().inverse();

        let proj = Mat4::perspective_rh(camera.fov, aspect_ratio, camera.near, camera.far);

        let mut vulkan_proj = proj;
        vulkan_proj.y_axis.y *= -1.0;

        let view_projection = vulkan_proj * view;

        render_world.insert_resource(ExtractedView {
            view_projection,
            camera_position: transform.translation,
        });
    }
}

pub fn extract_meshes_system(main_world: &mut World, render_world: &mut World) {
    let mut extracted_meshes = Vec::new();

    let mut query = main_world.query::<(&Transform, &Mesh)>();
    for (transform, mesh) in query.iter(main_world) {
        extracted_meshes.push(ExtractedMesh {
            transform: transform.compute_matrix(),
            opaque_vertices: mesh.opaque_vertices.clone(),
            transparent_vertices: mesh.transparent_vertices.clone(),
        });
    }

    render_world.insert_resource(ExtractedMeshes {
        meshes: extracted_meshes,
    })
}
