use crate::prelude::*;
use crate::render_bridge::RenderBridgePlugin;
use crate::voxel::VoxelDomainPlugin;

pub struct VoxelCorePlugin;

impl Plugin for VoxelCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((VoxelDomainPlugin, RenderBridgePlugin));
    }
}
