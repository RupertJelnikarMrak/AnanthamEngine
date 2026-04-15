mod camera;

pub use camera::Camera;

use crate::app::{App, Plugin, Startup};
use bevy_transform::TransformPlugin;
use camera::spawn_initial_camera;

pub struct SpatialDomainPlugin;

impl Plugin for SpatialDomainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TransformPlugin);

        app.add_systems(Startup, spawn_initial_camera);
    }
}
