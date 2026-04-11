use bevy_ecs::prelude::Component;
use glam::IVec3;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Component, Clone, Copy, PartialEq, Hash)]
pub struct ChunkCoord(pub IVec3);

#[derive(Component)]
pub struct Remesh;

#[derive(Component)]
pub struct Chunk {
    pub voxels: Box<[u16; CHUNK_VOLUME]>,
}

impl Chunk {
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }

    pub fn empty() -> Self {
        Self {
            voxels: vec![0; CHUNK_VOLUME].into_boxed_slice().try_into().unwrap(),
        }
    }
}
