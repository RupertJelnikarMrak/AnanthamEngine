use bevy_ecs::prelude::{Component, Resource};
use glam::{Mat4, Vec4};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec4,
    pub color: Vec4,
}

#[derive(Component, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
}

#[derive(Resource, Clone, Copy)]
pub struct ExtractedView {
    pub view_projection: Mat4,
}

pub struct ExtractedMesh {
    pub transform: Mat4,
    pub vertices: Vec<Vertex>,
}

#[derive(Resource, Default)]
pub struct ExtractedMeshes {
    pub meshes: Vec<ExtractedMesh>,
}
