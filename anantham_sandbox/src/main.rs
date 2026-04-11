use anantham_core::{
    App, BlockAttributes, BlockRegistry, Camera, EngineRunner, Input, LogPlugin, Transform,
    chunk_meshing_system, extract_camera_system, extract_meshes_system,
    voxel::{
        ChunkManager,
        chunk::{CHUNK_SIZE, Chunk, ChunkCoord, Remesh},
    },
};
use anantham_render::VoxelRenderPlugin;
use bevy_ecs::prelude::*;
use glam::{IVec3, Quat, Vec3, Vec4};
use noise::{Fbm, NoiseFn, Perlin};
use std::collections::HashSet;
use winit::keyboard::KeyCode;

fn camera_movement_system(
    mut input: ResMut<Input>,
    mut query: Query<(&mut Transform, &mut Camera)>,
) {
    let mut move_speed = 0.2;
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
        if input.pressed(KeyCode::ControlLeft) {
            velocity -= up;
        }
        if input.pressed(KeyCode::ShiftLeft) {
            move_speed = 0.5;
        }

        if velocity.length_squared() > 0.0 {
            transform.translation += velocity.normalize() * move_speed;
        }
    }
}

pub fn infinite_world_gen_system(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    let fbm = Fbm::<Perlin>::new(0);
    let render_distance = 3; // 3 chunks in every direction (7x7 grid)

    let mut chunks_in_range = HashSet::new();

    for camera_transform in camera_query.iter() {
        let camera_pos = camera_transform.translation;

        let current_chunk_x = (camera_pos.x / CHUNK_SIZE as f32).floor() as i32;
        let current_chunk_z = (camera_pos.z / CHUNK_SIZE as f32).floor() as i32;

        let mut newly_spawned = Vec::new();

        for dx in -render_distance..=render_distance {
            for dz in -render_distance..=render_distance {
                let chunk_coord = IVec3::new(current_chunk_x + dx, 0, current_chunk_z + dz);
                chunks_in_range.insert(chunk_coord);

                chunk_manager
                    .active_chunks
                    .entry(chunk_coord)
                    .or_insert_with(|| {
                        let mut chunk = Chunk::empty();

                        let world_offset_x = chunk_coord.x * CHUNK_SIZE as i32;
                        let world_offset_z = chunk_coord.z * CHUNK_SIZE as i32;

                        for x in 0..CHUNK_SIZE {
                            for z in 0..CHUNK_SIZE {
                                let world_x = world_offset_x + x as i32;
                                let world_z = world_offset_z + z as i32;

                                // Sample the noise
                                let noise_val =
                                    fbm.get([world_x as f64 * 0.05, world_z as f64 * 0.05]);
                                let terrain_height = ((noise_val + 1.0) * 8.0) as i32 + 4;
                                let sea_level = 10;

                                for y in 0..CHUNK_SIZE {
                                    let world_y = y as i32;

                                    if world_y < terrain_height {
                                        chunk.voxels[Chunk::index(x, y, z)] = 1; // Stone
                                    } else if world_y <= sea_level {
                                        chunk.voxels[Chunk::index(x, y, z)] = 2; // Glass
                                    }
                                }
                            }
                        }

                        newly_spawned.push(chunk_coord);

                        // Spawn and return the Entity ID directly into the HashMap
                        commands
                            .spawn((
                                chunk,
                                ChunkCoord(chunk_coord),
                                Remesh,
                                Transform {
                                    translation: Vec3::new(
                                        world_offset_x as f32,
                                        0.0,
                                        world_offset_z as f32,
                                    ),
                                    rotation: glam::Quat::IDENTITY,
                                    scale: Vec3::ONE,
                                },
                            ))
                            .id()
                    });
            }
        }

        for coord in newly_spawned {
            let neighbors = [
                IVec3::new(coord.x - 1, 0, coord.z),
                IVec3::new(coord.x + 1, 0, coord.z),
                IVec3::new(coord.x, 0, coord.z - 1),
                IVec3::new(coord.x, 0, coord.z + 1),
            ];

            for n_coord in neighbors {
                if let Some(&n_entity) = chunk_manager.active_chunks.get(&n_coord) {
                    commands.entity(n_entity).insert(Remesh);
                }
            }
        }
    }

    // Despawn chunks that have fallen out of range
    chunk_manager.active_chunks.retain(|coord, entity| {
        if chunks_in_range.contains(coord) {
            true // Keep it
        } else {
            commands.entity(*entity).despawn(); // Destroy it
            false // Remove from HashMap
        }
    });
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

    registry.register(
        "core:stone",
        BlockAttributes {
            is_transparent: false,
            color: Vec4::new(0.5, 0.5, 0.5, 1.0),
        },
    );

    registry.register(
        "core:glass",
        BlockAttributes {
            is_transparent: true,
            color: Vec4::new(0.8, 0.9, 1.0, 0.5),
        },
    );

    app.main_world.insert_resource(registry);
    app.main_world.insert_resource(ChunkManager::default());

    // 2. Register Systems
    app.main_schedule.add_systems(
        (
            camera_movement_system,
            infinite_world_gen_system,
            ApplyDeferred,
            chunk_meshing_system,
        )
            .chain(),
    );

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

    let runner = EngineRunner::new(app);
    runner.run()?;

    Ok(())
}
