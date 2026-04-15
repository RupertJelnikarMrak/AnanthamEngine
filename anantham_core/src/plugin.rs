use crate::app::{App, Plugin};
use crate::log::LogPlugin;
use crate::platform::PlatformPlugin;
use crate::render_bridge::RenderBridgePlugin;
use crate::spatial::SpatialDomainPlugin;
use crate::voxel::VoxelDomainPlugin;
use bevy_input::InputPlugin;

pub struct AnanthamCorePlugin;

impl Plugin for AnanthamCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TaskPoolPlugin::default(),
            LogPlugin,
            PlatformPlugin,
            InputPlugin,
            VoxelDomainPlugin,
            RenderBridgePlugin,
            SpatialDomainPlugin,
        ));
    }
}
