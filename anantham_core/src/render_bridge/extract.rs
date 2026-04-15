use crate::platform::prelude::*;
use crate::render_bridge::components::{ChunkMesh, ExtractedChunk, ExtractedMeshes, ExtractedView};
use crate::voxel::chunk::{CHUNK_SIZE, ChunkCoord};

pub fn extract_chunk_meshes(
    mut extracted_meshes: ResMut<ExtractedMeshes>,
    query: Query<(Entity, &ChunkCoord, &ChunkMesh)>,
    changed_query: Query<Entity, Changed<ChunkMesh>>,
) {
    extracted_meshes.chunks.clear();

    for (entity, coord, mesh) in query.iter() {
        // Build the world matrix based on the chunk's grid coordinate
        let transform = Mat4::from_translation(Vec3::new(
            (coord.0.x * CHUNK_SIZE as i32) as f32,
            (coord.0.y * CHUNK_SIZE as i32) as f32,
            (coord.0.z * CHUNK_SIZE as i32) as f32,
        ));

        // Only pack the heavy vertex data if the mesh was just created or modified this frame
        let is_changed = changed_query.contains(entity);

        extracted_meshes.chunks.push(ExtractedChunk {
            entity,
            transform,
            new_opaque_vertices: if is_changed {
                Some(mesh.opaque_vertices.clone())
            } else {
                None
            },
            new_transparent_vertices: if is_changed {
                Some(mesh.transparent_vertices.clone())
            } else {
                None
            },
        });
    }
}

pub fn extract_camera_view(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &Transform)>,
    extracted_view: Option<ResMut<ExtractedView>>,
) {
    let (camera, transform) = match camera_query.single() {
        Ok(q) => q,
        Err(err) => {
            tracing::error!(
                "Error querying the camera. Should always be only one instance: {}",
                err
            );
            return;
        }
    };
    let aspect_ratio = match window_query.single() {
        Ok(win) => {
            let w = win.resolution.width();
            let h = win.resolution.height();

            if h > 0.0 { w / h } else { 19.0 / 9.0 }
        }
        Err(err) => {
            tracing::error!(
                "Error querying the window. Should always be only one instance: {}",
                err
            );
            return;
        }
    };

    // 3. Compute the Projection Matrix
    let mut projection = Mat4::perspective_rh(camera.fov, aspect_ratio, camera.near, camera.far);

    // VULKAN FIX Standard perspective matrices assume Y points UP (OpenGL style).
    // Vulkan's clip space has Y pointing DOWN. We must invert the Y-axis of the projection.
    projection.y_axis.y *= -1.0;

    // 4. Compute the View Matrix
    let forward = transform.rotation * -Vec3::Z;
    let up = transform.rotation * Vec3::Y;
    let view = Mat4::look_to_rh(transform.translation, forward, up);

    // 5. Cache the View-Projection matrix for the GPU push constants
    let view_projection = projection * view;

    // 6. Push to the Render World using your actual struct fields
    if let Some(mut current_view) = extracted_view {
        current_view.view_projection = view_projection;
        current_view.camera_position = transform.translation;
    } else {
        // First frame: initialize the resource
        commands.insert_resource(ExtractedView {
            view_projection,
            camera_position: transform.translation,
        });
    }
}
