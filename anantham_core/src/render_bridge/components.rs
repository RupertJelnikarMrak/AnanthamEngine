use bevy_ecs::prelude::{Component, Resource};
use glam::{Mat4, Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec4,
    pub color: Vec4,
    pub normal: Vec4,
}

#[derive(Component, Clone)]
pub struct Mesh {
    pub opaque_vertices: Vec<Vertex>,
    pub transparent_vertices: Vec<Vertex>,
}

#[derive(Resource, Clone, Copy)]
pub struct ExtractedView {
    pub view_projection: Mat4,
    pub camera_position: Vec3,
}

pub struct ExtractedMesh {
    pub transform: Mat4,
    pub opaque_vertices: Vec<Vertex>,
    pub transparent_vertices: Vec<Vertex>,
}

#[derive(Resource, Default)]
pub struct ExtractedMeshes {
    pub meshes: Vec<ExtractedMesh>,
}
