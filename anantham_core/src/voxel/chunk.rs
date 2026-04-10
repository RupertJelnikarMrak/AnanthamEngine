use bevy_ecs::prelude::Component;
use glam::IVec3;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Component, Clone, Copy, PartialEq, Hash)]
pub struct ChunkCood(pub IVec3);

#[derive(Component)]
pub struct Chunk {
    pub voxels: Box<[u16; CHUNK_VOLUME]>,
}

impl Chunk {
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }
}
