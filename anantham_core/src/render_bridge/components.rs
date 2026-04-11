use bevy_ecs::prelude::{Component, Resource};
use glam::{Mat4, Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec4,
    pub color: Vec4,
    pub normal: Vec4,
}

/// Attached to a chunk entity by the background meshing thread.
#[derive(Component, Clone)]
pub struct ChunkMesh {
    pub opaque_vertices: Vec<Vertex>,
    pub transparent_vertices: Vec<Vertex>,
}

/// The extracted data required by the Render World to draw a single chunk.
#[derive(Clone)]
pub struct ExtractedChunk {
    pub entity: bevy_ecs::entity::Entity,
    pub transform: Mat4,
    // We wrap these in an Option. If Some, the Render World must upload them to the GeometryArena.
    // If None, the Render World just uses the transform and its existing arena offset.
    pub new_opaque_vertices: Option<Vec<Vertex>>,
    pub new_transparent_vertices: Option<Vec<Vertex>>,
}

/// The payload delivered to the RenderSchedule every frame.
#[derive(Resource, Default)]
pub struct ExtractedMeshes {
    pub chunks: Vec<ExtractedChunk>,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct ExtractedView {
    pub view_projection: Mat4,
    pub camera_position: Vec3,
}
