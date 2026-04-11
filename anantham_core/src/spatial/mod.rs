pub mod camera;
pub mod transform;

use crate::prelude::*;
use camera::Camera;
use transform::Transform;

pub struct SpatialDomainPlugin;

impl Plugin for SpatialDomainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_initial_camera);
    }
}

fn spawn_initial_camera(mut commands: Commands) {
    commands.spawn((
        Camera::default(),
        Transform {
            translation: glam::Vec3::new(16.0, 40.0, 16.0),
            ..Default::default()
        },
    ));
}
