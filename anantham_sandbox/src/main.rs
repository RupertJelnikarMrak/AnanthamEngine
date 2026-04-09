use anantham_core::{
    App, EngineRunner, LogPlugin,
    components::{Camera, Transform},
    extract::extract_camera_system,
};
use anantham_render::VoxelRenderPlugin;
use glam::{Quat, Vec3};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    app.add_plugin(LogPlugin);
    app.add_plugin(VoxelRenderPlugin);

    app.add_extract_system(extract_camera_system);

    app.main_world.spawn((
        Camera {
            fov: 1.57, // ~90 degrees
            near: 0.1,
            far: 1000.0,
        },
        Transform {
            translation: Vec3::new(0.0, 0.0, 2.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    ));

    let runner = EngineRunner::new(app);
    runner.run()?;

    Ok(())
}
