use anantham_core::{
    App, Camera, EngineRunner, Input, LogPlugin, Transform, extract_camera_system,
};
use anantham_render::VoxelRenderPlugin;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;

fn camera_movement_system(input: Res<Input>, mut query: Query<&mut Transform, With<Camera>>) {
    let speed = 0.05; // Quick hardcoded speed for the MVP

    for mut transform in &mut query {
        let mut velocity = Vec3::ZERO;

        if input.pressed(KeyCode::KeyW) {
            velocity.z -= speed;
        }
        if input.pressed(KeyCode::KeyS) {
            velocity.z += speed;
        }
        if input.pressed(KeyCode::KeyA) {
            velocity.x -= speed;
        }
        if input.pressed(KeyCode::KeyD) {
            velocity.x += speed;
        }
        if input.pressed(KeyCode::Space) {
            velocity.y += speed;
        }
        if input.pressed(KeyCode::ShiftLeft) {
            velocity.y -= speed;
        }

        transform.translation += velocity;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    app.add_plugin(LogPlugin);
    app.add_plugin(VoxelRenderPlugin);

    app.main_schedule.add_systems(camera_movement_system);
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
