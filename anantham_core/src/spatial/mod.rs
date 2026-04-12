pub mod camera;
pub mod transform;

pub mod prelude {
    pub use super::camera::Camera;
    pub use super::transform::Transform;
}

use crate::plugin_prelude::*;
use crate::spatial::camera::spawn_initial_camera;

pub struct SpatialDomainPlugin;

impl Plugin for SpatialDomainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_initial_camera);
    }
}
