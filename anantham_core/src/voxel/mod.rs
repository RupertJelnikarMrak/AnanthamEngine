pub mod chunk;
pub mod mesher;
pub mod registry;

pub mod prelude {
    pub use super::chunk::*;
    pub use super::registry::*;
}

use crate::app::{App, Plugin};

pub struct VoxelDomainPlugin;

impl Plugin for VoxelDomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<registry::BlockRegistry>();
        app.init_resource::<chunk::ChunkManager>();

        app.add_systems(
            Update,
            (mesher::spawn_meshing_tasks, mesher::poll_meshing_tasks),
        );
    }
}
