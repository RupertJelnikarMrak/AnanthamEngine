use anantham_core::{
    App, BlockAttributes, BlockRegistry, Camera, EngineRunner, Input, LogPlugin, Transform,
    chunk_meshing_system, extract_camera_system, extract_meshes_system,
    voxel::chunk::{CHUNK_VOLUME, Chunk},
};
use anantham_render::VoxelRenderPlugin;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3, Vec4};
use winit::keyboard::KeyCode;

fn camera_movement_system(
    mut input: ResMut<Input>,
    mut query: Query<(&mut Transform, &mut Camera)>,
) {
    let move_speed = 0.05;
    let look_speed = 0.001;

    let mouse_delta = input.take_mouse_delta();

    for (mut transform, mut camera) in &mut query {
        camera.yaw -= mouse_delta.x * look_speed;
        camera.pitch -= mouse_delta.y * look_speed;

        camera.pitch = camera.pitch.clamp(-1.5, 1.5);
        transform.rotation = Quat::from_euler(glam::EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);

        let mut velocity = Vec3::ZERO;

        let forward = transform.rotation * -Vec3::Z;
        let right = transform.rotation * Vec3::X;
        let up = Vec3::Y;

        if input.pressed(KeyCode::KeyW) {
            velocity += forward;
        }
        if input.pressed(KeyCode::KeyS) {
            velocity -= forward;
        }
        if input.pressed(KeyCode::KeyA) {
            velocity -= right;
        }
        if input.pressed(KeyCode::KeyD) {
            velocity += right;
        }
        if input.pressed(KeyCode::Space) {
            velocity += up;
        }
        if input.pressed(KeyCode::ShiftLeft) {
            velocity -= up;
        }

        if velocity.length_squared() > 0.0 {
            transform.translation += velocity.normalize() * move_speed;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    app.add_plugin(LogPlugin);
    app.add_plugin(VoxelRenderPlugin);

    let mut registry = BlockRegistry::new();

    registry.register(
        "core:air",
        BlockAttributes {
            is_transparent: true,
            color: Vec4::ZERO,
        },
    );

    let stone_id = registry.register(
        "core:stone",
        BlockAttributes {
            is_transparent: false,
            color: Vec4::new(0.5, 0.5, 0.5, 1.0),
        },
    );

    let glass_id = registry.register(
        "core:glass",
        BlockAttributes {
            is_transparent: true,
            color: Vec4::new(0.8, 0.9, 1.0, 0.5),
        },
    );

    app.main_world.insert_resource(registry);

    // 2. Register Systems
    app.main_schedule.add_systems(camera_movement_system);
    app.main_schedule.add_systems(chunk_meshing_system);
    app.add_extract_system(extract_camera_system);
    app.add_extract_system(extract_meshes_system);

    app.main_world.spawn((
        Camera {
            fov: 1.57, // ~90 degrees
            near: 0.1,
            far: 1000.0,
            pitch: 0.0,
            yaw: 0.0,
        },
        Transform {
            translation: Vec3::new(16.5, 17.0, 18.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    ));

    // 3. Spawn a Test Chunk!
    let mut voxels = Box::new([0; CHUNK_VOLUME]);

    // Fill the bottom half with stone, and put a few glass blocks on top
    for x in 0..32 {
        for y in 0..16 {
            for z in 0..32 {
                voxels[Chunk::index(x, y, z)] = stone_id;
            }
        }
    }
    voxels[Chunk::index(16, 16, 16)] = glass_id;
    voxels[Chunk::index(17, 16, 16)] = glass_id;

    app.main_world.spawn((
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Chunk { voxels },
    ));

    let runner = EngineRunner::new(app);
    runner.run()?;

    Ok(())
}
