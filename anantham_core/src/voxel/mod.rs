pub mod chunk;
pub mod mesher;
pub mod registry;

use bevy_ecs::prelude::{Entity, Resource};
use glam::IVec3;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct ChunkManager {
    pub active_chunks: HashMap<IVec3, Entity>,
}
