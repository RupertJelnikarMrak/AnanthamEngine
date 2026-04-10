use anantham_core::{
    App, Camera, EngineRunner, Input, LogPlugin, Transform, extract_camera_system,
    extract_meshes_system,
    render_bridge::components::{Mesh, Vertex},
};
use anantham_render::VoxelRenderPlugin;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3, Vec4};
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
    app.add_extract_system(extract_meshes_system);

    let triangle_vertices = vec![
        Vertex {
            position: Vec4::new(0.0, 0.5, 0.0, 1.0),
            color: Vec4::new(1.0, 0.0, 0.0, 1.0),
        },
        Vertex {
            position: Vec4::new(0.5, -0.5, 0.0, 1.0),
            color: Vec4::new(0.0, 1.0, 0.0, 1.0),
        },
        Vertex {
            position: Vec4::new(-0.5, -0.5, 0.0, 1.0),
            color: Vec4::new(0.0, 0.0, 1.0, 1.0),
        },
    ];

    for x in -5..5 {
        for y in -5..5 {
            app.main_world.spawn((
                Transform {
                    translation: Vec3::new(x as f32 * 1.5, y as f32 * 1.5, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                Mesh {
                    vertices: triangle_vertices.clone(),
                },
            ));
        }
    }

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
