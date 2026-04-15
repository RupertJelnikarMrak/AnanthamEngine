use anantham_core::platform::prelude::MouseMotion;
use anantham_core::prelude::*;
use anantham_core::voxel::chunk::{
    CHUNK_SIZE, CHUNK_VOLUME, ChunkCoord, ChunkManager, NeedsMeshing, VoxelData,
};
use anantham_core::voxel::registry::{BlockAttributes, BlockRegistry};
use anantham_render::RenderBackendPlugin;
use noise::{Fbm, NoiseFn, Perlin};
use std::collections::HashSet;
use std::sync::Arc;

#[inline]
fn voxel_index(x: usize, y: usize, z: usize) -> usize {
    x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
}

fn camera_movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut Camera)>,
) {
    let mut move_speed = 0.2;
    let look_speed = 0.001;

    // Note: anantham_core's handler.rs currently forwards Keyboards, but not MouseMotion yet.
    // Replace this stub with an EventReader<MouseMotion> once added to the platform handler.
    let mouse_delta = Vec2::ZERO;
    for event in mouse_events.read() {
        mouse_delta += event.delta;
    }

    for (mut transform, mut camera) in &mut query {
        camera.yaw -= mouse_delta.x * look_speed;
        camera.pitch -= mouse_delta.y * look_speed;

        camera.pitch = camera.pitch.clamp(-1.5, 1.5);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);

        let mut velocity = Vec3::ZERO;

        let forward = transform.rotation * -Vec3::Z;
        let right = transform.rotation * Vec3::X;
        let up = Vec3::Y;

        if keyboard.pressed(KeyCode::KeyW) {
            velocity += forward;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            velocity -= forward;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            velocity -= right;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            velocity += right;
        }
        if keyboard.pressed(KeyCode::Space) {
            velocity += up;
        }
        if keyboard.pressed(KeyCode::ControlLeft) {
            velocity -= up;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) {
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
                let coord_key = ChunkCoord(chunk_coord);

                chunks_in_range.insert(coord_key);

                // Use the Entry API to avoid double-hashing the coordinate
                chunk_manager
                    .active_chunks
                    .entry(coord_key)
                    .or_insert_with(|| {
                        let mut voxels = [0u16; CHUNK_VOLUME];

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
                                    let idx = voxel_index(x, y, z);

                                    if world_y < terrain_height {
                                        voxels[idx] = 1; // Stone
                                    } else if world_y <= sea_level {
                                        voxels[idx] = 2; // Glass
                                    }
                                }
                            }
                        }

                        newly_spawned.push(chunk_coord);

                        // Spawn into the ECS and return the Entity ID to be inserted into the HashMap
                        commands
                            .spawn((
                                VoxelData(Arc::new(voxels)),
                                coord_key,
                                NeedsMeshing,
                                Transform {
                                    translation: Vec3::new(
                                        world_offset_x as f32,
                                        0.0,
                                        world_offset_z as f32,
                                    ),
                                    ..Default::default()
                                },
                            ))
                            .id()
                    });
            }
        }

        // Trigger meshing on neighbors of newly generated chunks
        for coord in newly_spawned {
            let neighbors = [
                ChunkCoord(IVec3::new(coord.x - 1, 0, coord.z)),
                ChunkCoord(IVec3::new(coord.x + 1, 0, coord.z)),
                ChunkCoord(IVec3::new(coord.x, 0, coord.z - 1)),
                ChunkCoord(IVec3::new(coord.x, 0, coord.z + 1)),
            ];

            for n_coord in neighbors {
                if let Some(&n_entity) = chunk_manager.active_chunks.get(&n_coord) {
                    commands.entity(n_entity).insert(NeedsMeshing);
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

// Data initialization runs once at boot
fn setup_blocks(mut registry: ResMut<BlockRegistry>) {
    // Note: "air" (ID 0) is natively registered by BlockRegistry::new()

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
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera {
            fov: 1.57, // ~90 degrees
            near: 0.1,
            far: 1000.0,
            pitch: 0.0,
            yaw: 0.0,
        },
        Transform {
            // Spawning you slightly above the sea level so you don't clip into the ground
            translation: Vec3::new(16.5, 20.0, 18.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    ));
}

fn main() -> AppExit {
    let mut app = App::new();

    // 1. Add Engine Plugins (Handles all scheduling, meshing threads, and Vulkan)
    app.add_plugins(AnanthamCorePlugin);
    app.add_plugins(RenderBackendPlugin);

    // 2. Register User Systems
    app.add_systems(Startup, setup_blocks);
    app.add_systems(Startup, setup_camera);
    app.add_systems(Update, (camera_movement_system, infinite_world_gen_system));

    // 3. Hand control over to the platform runner
    app.run()
}
