use crate::ecs::{Component, Resource};
use crate::render_bridge::components::Vertex;
use std::collections::HashMap;
use std::sync::Arc;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord(pub IVec3);

#[derive(Resource, Default)]
pub struct ChunkManager {
    pub active_chunks: HashMap<ChunkCoord, Entity>,
}

#[derive(Component, Clone)]
pub struct VoxelData(pub Arc<[u16; CHUNK_VOLUME]>);

impl VoxelData {
    pub fn empty() -> Self {
        Self(Arc::new([0; CHUNK_VOLUME]))
    }
}

#[derive(Component)]
pub struct NeedsMeshing;

#[derive(Component)]
pub struct MeshingTask(pub Task<MeshedChunkData>);

pub struct MeshedChunkData {
    pub opaque_vertices: Vec<Vertex>,
    pub transparent_vertices: Vec<Vertex>,
}
