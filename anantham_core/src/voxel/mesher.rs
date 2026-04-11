use crate::prelude::*;
use crate::render_bridge::components::{ChunkMesh, Vertex};
use crate::voxel::chunk::{
    CHUNK_SIZE, ChunkCoord, ChunkManager, MeshedChunkData, MeshingTask, NeedsMeshing, VoxelData,
};
use crate::voxel::registry::{BlockAttributes, BlockRegistry};
use bevy_tasks::AsyncComputeTaskPool;
use futures_lite::future;
use glam::{Vec3, Vec4};
use std::sync::Arc;

#[inline]
fn voxel_index(x: usize, y: usize, z: usize) -> usize {
    x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
}

pub fn spawn_meshing_tasks(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    chunk_manager: Res<ChunkManager>,
    chunk_query: Query<&VoxelData>,
    remesh_query: Query<(Entity, &ChunkCoord), With<NeedsMeshing>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    let attributes_arc = registry.attributes.clone();

    for (entity, coord) in remesh_query.iter() {
        let Ok(voxel_data) = chunk_query.get(entity) else {
            continue;
        };

        let get_neighbor = |dx: i32,
                            dy: i32,
                            dz: i32|
         -> Option<Arc<[u16; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE]>> {
            let mut n_coord = coord.0;
            n_coord.x += dx;
            n_coord.y += dy;
            n_coord.z += dz;

            if let Some(&n_entity) = chunk_manager.active_chunks.get(&ChunkCoord(n_coord))
                && let Ok(n_data) = chunk_query.get(n_entity)
            {
                return Some(n_data.0.clone());
            }

            None
        };

        let neighbors = [
            get_neighbor(1, 0, 0),
            get_neighbor(-1, 0, 0),
            get_neighbor(0, 1, 0),
            get_neighbor(0, -1, 0),
            get_neighbor(0, 0, 1),
            get_neighbor(0, 0, -1),
        ];

        let voxels_clone = voxel_data.0.clone();
        let attrs_clone = attributes_arc.clone();

        let task = thread_pool.spawn(async move {
            let (opaque, transparent) =
                mesh_chunk_internal(&voxels_clone, &neighbors, &attrs_clone);
            MeshedChunkData {
                opaque_vertices: opaque,
                transparent_vertices: transparent,
            }
        });

        commands
            .entity(entity)
            .remove::<NeedsMeshing>()
            .insert(MeshingTask(task));
    }
}

pub fn poll_meshing_tasks(mut commands: Commands, mut query: Query<(Entity, &mut MeshingTask)>) {
    for (entity, mut task) in &mut query {
        if let Some(meshed_data) = future::block_on(future::poll_once(&mut task.0)) {
            commands
                .entity(entity)
                .remove::<MeshingTask>()
                .insert(ChunkMesh {
                    opaque_vertices: meshed_data.opaque_vertices,
                    transparent_vertices: meshed_data.transparent_vertices,
                });
        }
    }
}

fn mesh_chunk_internal(
    voxels: &[u16; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
    neighbors: &[Option<Arc<[u16; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE]>>; 6],
    attributes: &[BlockAttributes], // Directly accepts the slice from the Arc
) -> (Vec<Vertex>, Vec<Vertex>) {
    let mut opaque_vertices = Vec::new();
    let mut transparent_vertices = Vec::new();

    let is_face_visible = |voxel_id: u16, nx: i32, ny: i32, nz: i32| -> bool {
        let neighbor_id = if nx >= 0
            && nx < CHUNK_SIZE as i32
            && ny >= 0
            && ny < CHUNK_SIZE as i32
            && nz >= 0
            && nz < CHUNK_SIZE as i32
        {
            voxels[voxel_index(nx as usize, ny as usize, nz as usize)]
        } else {
            let (n_idx, lx, ly, lz) = if nx < 0 {
                (1, nx + CHUNK_SIZE as i32, ny, nz)
            } else if nx >= CHUNK_SIZE as i32 {
                (0, nx - CHUNK_SIZE as i32, ny, nz)
            } else if ny < 0 {
                (3, nx, ny + CHUNK_SIZE as i32, nz)
            } else if ny >= CHUNK_SIZE as i32 {
                (2, nx, ny - CHUNK_SIZE as i32, nz)
            } else if nz < 0 {
                (5, nx, ny, nz + CHUNK_SIZE as i32)
            } else {
                (4, nx, ny, nz - CHUNK_SIZE as i32)
            };

            if let Some(n_voxels) = &neighbors[n_idx] {
                n_voxels[voxel_index(lx as usize, ly as usize, lz as usize)]
            } else {
                0
            }
        };

        if voxel_id == neighbor_id {
            return false;
        }

        attributes
            .get(neighbor_id as usize)
            .unwrap_or(&attributes[0])
            .is_transparent
    };

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let voxel_id = voxels[voxel_index(x, y, z)];
                if voxel_id == 0 {
                    continue;
                }

                let attrs = attributes.get(voxel_id as usize).unwrap_or(&attributes[0]);
                let position = Vec3::new(x as f32, y as f32, z as f32);
                let color = attrs.color;

                let target_vertices = if attrs.is_transparent {
                    &mut transparent_vertices
                } else {
                    &mut opaque_vertices
                };

                let ix = x as i32;
                let iy = y as i32;
                let iz = z as i32;

                if is_face_visible(voxel_id, ix, iy + 1, iz) {
                    add_face_top(target_vertices, position, color);
                }
                if is_face_visible(voxel_id, ix, iy - 1, iz) {
                    add_face_bottom(target_vertices, position, color);
                }
                if is_face_visible(voxel_id, ix + 1, iy, iz) {
                    add_face_right(target_vertices, position, color);
                }
                if is_face_visible(voxel_id, ix - 1, iy, iz) {
                    add_face_left(target_vertices, position, color);
                }
                if is_face_visible(voxel_id, ix, iy, iz + 1) {
                    add_face_front(target_vertices, position, color);
                }
                if is_face_visible(voxel_id, ix, iy, iz - 1) {
                    add_face_back(target_vertices, position, color);
                }
            }
        }
    }
    (opaque_vertices, transparent_vertices)
}

#[inline]
fn push_quad(
    vertices: &mut Vec<Vertex>,
    p0: Vec4,
    p1: Vec4,
    p2: Vec4,
    p3: Vec4,
    color: Vec4,
    normal: Vec4,
) {
    vertices.push(Vertex {
        position: p0,
        color,
        normal,
    });
    vertices.push(Vertex {
        position: p1,
        color,
        normal,
    });
    vertices.push(Vertex {
        position: p2,
        color,
        normal,
    });
    vertices.push(Vertex {
        position: p2,
        color,
        normal,
    });
    vertices.push(Vertex {
        position: p3,
        color,
        normal,
    });
    vertices.push(Vertex {
        position: p0,
        color,
        normal,
    });
}

fn add_face_top(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x, pos.y + 1.0, pos.z + 1.0, 1.0);
    let p1 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0, 1.0);
    let p2 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z, 1.0);
    let p3 = Vec4::new(pos.x, pos.y + 1.0, pos.z, 1.0);
    push_quad(
        vertices,
        p0,
        p1,
        p2,
        p3,
        color,
        Vec4::new(0.0, 1.0, 0.0, 0.0),
    );
}

fn add_face_bottom(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x, pos.y, pos.z, 1.0);
    let p1 = Vec4::new(pos.x + 1.0, pos.y, pos.z, 1.0);
    let p2 = Vec4::new(pos.x + 1.0, pos.y, pos.z + 1.0, 1.0);
    let p3 = Vec4::new(pos.x, pos.y, pos.z + 1.0, 1.0);
    push_quad(
        vertices,
        p0,
        p1,
        p2,
        p3,
        color,
        Vec4::new(0.0, -1.0, 0.0, 0.0),
    );
}

fn add_face_right(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x + 1.0, pos.y, pos.z + 1.0, 1.0);
    let p1 = Vec4::new(pos.x + 1.0, pos.y, pos.z, 1.0);
    let p2 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z, 1.0);
    let p3 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0, 1.0);
    push_quad(
        vertices,
        p0,
        p1,
        p2,
        p3,
        color,
        Vec4::new(1.0, 0.0, 0.0, 0.0),
    );
}

fn add_face_left(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x, pos.y, pos.z, 1.0);
    let p1 = Vec4::new(pos.x, pos.y, pos.z + 1.0, 1.0);
    let p2 = Vec4::new(pos.x, pos.y + 1.0, pos.z + 1.0, 1.0);
    let p3 = Vec4::new(pos.x, pos.y + 1.0, pos.z, 1.0);
    push_quad(
        vertices,
        p0,
        p1,
        p2,
        p3,
        color,
        Vec4::new(-1.0, 0.0, 0.0, 0.0),
    );
}

fn add_face_front(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x, pos.y, pos.z + 1.0, 1.0);
    let p1 = Vec4::new(pos.x + 1.0, pos.y, pos.z + 1.0, 1.0);
    let p2 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0, 1.0);
    let p3 = Vec4::new(pos.x, pos.y + 1.0, pos.z + 1.0, 1.0);
    push_quad(
        vertices,
        p0,
        p1,
        p2,
        p3,
        color,
        Vec4::new(0.0, 0.0, 1.0, 0.0),
    );
}

fn add_face_back(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x + 1.0, pos.y, pos.z, 1.0);
    let p1 = Vec4::new(pos.x, pos.y, pos.z, 1.0);
    let p2 = Vec4::new(pos.x, pos.y + 1.0, pos.z, 1.0);
    let p3 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z, 1.0);
    push_quad(
        vertices,
        p0,
        p1,
        p2,
        p3,
        color,
        Vec4::new(0.0, 0.0, -1.0, 0.0),
    );
}
