use crate::render_bridge::components::{Mesh, Vertex};
use crate::voxel::ChunkManager;
use crate::voxel::chunk::{CHUNK_SIZE, Chunk, ChunkCoord, Remesh};
use crate::voxel::registry::BlockRegistry;
use bevy_ecs::prelude::*;
use glam::{Vec3, Vec4};

pub fn chunk_meshing_system(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    chunk_manager: Res<ChunkManager>,
    chunk_query: Query<&Chunk>, // Used to look up neighbor data
    remesh_query: Query<(Entity, &ChunkCoord), With<Remesh>>,
) {
    for (entity, coord) in remesh_query.iter() {
        let Ok(chunk) = chunk_query.get(entity) else {
            continue;
        };

        let mut opaque_vertices = Vec::new();
        let mut transparent_vertices = Vec::new();

        // Cross-Chunk visibility helper
        let is_face_visible = |voxel_id: u16, nx: i32, ny: i32, nz: i32| -> bool {
            let neighbor_id = if nx >= 0
                && nx < CHUNK_SIZE as i32
                && ny >= 0
                && ny < CHUNK_SIZE as i32
                && nz >= 0
                && nz < CHUNK_SIZE as i32
            {
                // Inside the current chunk
                chunk.voxels[Chunk::index(nx as usize, ny as usize, nz as usize)]
            } else {
                // Across the chunk border! Calculate which neighbor to ask.
                let mut n_coord = coord.0;
                let mut lx = nx;
                let mut ly = ny;
                let mut lz = nz;

                if nx < 0 {
                    n_coord.x -= 1;
                    lx += CHUNK_SIZE as i32;
                } else if nx >= CHUNK_SIZE as i32 {
                    n_coord.x += 1;
                    lx -= CHUNK_SIZE as i32;
                }

                if ny < 0 {
                    n_coord.y -= 1;
                    ly += CHUNK_SIZE as i32;
                } else if ny >= CHUNK_SIZE as i32 {
                    n_coord.y += 1;
                    ly -= CHUNK_SIZE as i32;
                }

                if nz < 0 {
                    n_coord.z -= 1;
                    lz += CHUNK_SIZE as i32;
                } else if nz >= CHUNK_SIZE as i32 {
                    n_coord.z += 1;
                    lz -= CHUNK_SIZE as i32;
                }

                // Ask the ChunkManager if the neighbor exists
                if let Some(&n_entity) = chunk_manager.active_chunks.get(&n_coord) {
                    if let Ok(n_chunk) = chunk_query.get(n_entity) {
                        n_chunk.voxels[Chunk::index(lx as usize, ly as usize, lz as usize)]
                    } else {
                        0 // Entity exists but chunk not loaded
                    }
                } else {
                    0 // Chunk is entirely unloaded, treat the void as air so we see the edge of the world
                }
            };

            // If the blocks are identical (e.g., Glass touching Glass), hide the internal wall
            if voxel_id == neighbor_id {
                return false;
            }

            // Otherwise, draw the wall if the neighbor is transparent (Air or Glass)
            registry.get(neighbor_id).is_transparent
        };

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let voxel_id = chunk.voxels[Chunk::index(x, y, z)];

                    if voxel_id == 0 {
                        continue; // Skip Air
                    }

                    let attributes = registry.get(voxel_id);
                    let position = Vec3::new(x as f32, y as f32, z as f32);
                    let color = attributes.color;

                    let target_vertices = if attributes.is_transparent {
                        &mut transparent_vertices
                    } else {
                        &mut opaque_vertices
                    };

                    let ix = x as i32;
                    let iy = y as i32;
                    let iz = z as i32;

                    // The logic is now beautifully unified! No more hardcoded CHUNK_SIZE edge checks.
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

        commands.entity(entity).insert(Mesh {
            opaque_vertices,
            transparent_vertices,
        });
    }
}

// --- Face Generation Helpers ---
/// Pushes two triangles (6 vertices) to form a quad.
/// The winding order is 0 -> 1 -> 2 and 2 -> 3 -> 0.
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
    ); // -Y
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
    ); // +X
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
    ); // +Z
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
    ); // -Z
}
