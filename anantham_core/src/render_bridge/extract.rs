use crate::prelude::*;
use crate::render_bridge::components::{ChunkMesh, ExtractedChunk, ExtractedMeshes, ExtractedView};
use crate::voxel::chunk::{CHUNK_SIZE, ChunkCoord};
use glam::{Mat4, Vec3};

/// Extracts chunks from the Main World and prepares them for the Render World.
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

/// Extracts the camera position and projection matrix (Placeholder for your actual camera logic)
pub fn extract_camera_view(
    mut extracted_view: ResMut<ExtractedView>,
    // TODO: query: Query<(&Camera, &Transform)> // You will hook this up to your spatial domain later
) {
    // For now, we just pass an identity matrix so the engine compiles
    extracted_view.view_projection = Mat4::IDENTITY;
    extracted_view.camera_position = Vec3::ZERO;
}
