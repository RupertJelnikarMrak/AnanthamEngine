use crate::render_bridge::components::{Mesh, Vertex};
use crate::voxel::chunk::{CHUNK_SIZE, Chunk};
use crate::voxel::registry::BlockRegistry;
use bevy_ecs::prelude::*;
use glam::{Vec3, Vec4};

#[inline]
fn should_draw_face(voxel_id: u16, neighbor_id: u16, registry: &BlockRegistry) -> bool {
    if voxel_id == neighbor_id {
        return false;
    }

    registry.get(neighbor_id).is_transparent
}

pub fn chunk_meshing_system(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    query: Query<(Entity, &Chunk), Added<Chunk>>,
) {
    for (entity, chunk) in query.iter() {
        let mut opaque_vertices = Vec::new();
        let mut transparent_vertices = Vec::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let voxel_id = chunk.voxels[Chunk::index(x, y, z)];

                    // Skip Air
                    if voxel_id == 0 {
                        continue;
                    }

                    let attributes = registry.get(voxel_id);
                    let position = Vec3::new(x as f32, y as f32, z as f32);
                    let color = attributes.color;

                    let target_vertices = if attributes.is_transparent {
                        &mut transparent_vertices
                    } else {
                        &mut opaque_vertices
                    };

                    // Top (+Y)
                    if y == CHUNK_SIZE - 1
                        || should_draw_face(
                            voxel_id,
                            chunk.voxels[Chunk::index(x, y + 1, z)],
                            &registry,
                        )
                    {
                        add_face_top(target_vertices, position, color);
                    }
                    // Bottom (-Y)
                    if y == 0
                        || should_draw_face(
                            voxel_id,
                            chunk.voxels[Chunk::index(x, y - 1, z)],
                            &registry,
                        )
                    {
                        add_face_bottom(target_vertices, position, color);
                    }
                    // Right (+X)
                    if x == CHUNK_SIZE - 1
                        || should_draw_face(
                            voxel_id,
                            chunk.voxels[Chunk::index(x + 1, y, z)],
                            &registry,
                        )
                    {
                        add_face_right(target_vertices, position, color);
                    }
                    // Left (-X)
                    if x == 0
                        || should_draw_face(
                            voxel_id,
                            chunk.voxels[Chunk::index(x - 1, y, z)],
                            &registry,
                        )
                    {
                        add_face_left(target_vertices, position, color);
                    }
                    // Front (+Z)
                    if z == CHUNK_SIZE - 1
                        || should_draw_face(
                            voxel_id,
                            chunk.voxels[Chunk::index(x, y, z + 1)],
                            &registry,
                        )
                    {
                        add_face_front(target_vertices, position, color);
                    }
                    // Back (-Z)
                    if z == 0
                        || should_draw_face(
                            voxel_id,
                            chunk.voxels[Chunk::index(x, y, z - 1)],
                            &registry,
                        )
                    {
                        add_face_back(&mut opaque_vertices, position, color);
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
fn push_quad(vertices: &mut Vec<Vertex>, p0: Vec4, p1: Vec4, p2: Vec4, p3: Vec4, color: Vec4) {
    vertices.push(Vertex {
        position: p0,
        color,
    });
    vertices.push(Vertex {
        position: p1,
        color,
    });
    vertices.push(Vertex {
        position: p2,
        color,
    });

    vertices.push(Vertex {
        position: p2,
        color,
    });
    vertices.push(Vertex {
        position: p3,
        color,
    });
    vertices.push(Vertex {
        position: p0,
        color,
    });
}

fn add_face_top(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x, pos.y + 1.0, pos.z + 1.0, 1.0);
    let p1 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0, 1.0);
    let p2 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z, 1.0);
    let p3 = Vec4::new(pos.x, pos.y + 1.0, pos.z, 1.0);
    push_quad(vertices, p0, p1, p2, p3, color);
}

fn add_face_bottom(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x, pos.y, pos.z, 1.0);
    let p1 = Vec4::new(pos.x + 1.0, pos.y, pos.z, 1.0);
    let p2 = Vec4::new(pos.x + 1.0, pos.y, pos.z + 1.0, 1.0);
    let p3 = Vec4::new(pos.x, pos.y, pos.z + 1.0, 1.0);
    push_quad(vertices, p0, p1, p2, p3, color);
}

fn add_face_right(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x + 1.0, pos.y, pos.z + 1.0, 1.0);
    let p1 = Vec4::new(pos.x + 1.0, pos.y, pos.z, 1.0);
    let p2 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z, 1.0);
    let p3 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0, 1.0);
    push_quad(vertices, p0, p1, p2, p3, color);
}

fn add_face_left(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x, pos.y, pos.z, 1.0);
    let p1 = Vec4::new(pos.x, pos.y, pos.z + 1.0, 1.0);
    let p2 = Vec4::new(pos.x, pos.y + 1.0, pos.z + 1.0, 1.0);
    let p3 = Vec4::new(pos.x, pos.y + 1.0, pos.z, 1.0);
    push_quad(vertices, p0, p1, p2, p3, color);
}

fn add_face_front(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x, pos.y, pos.z + 1.0, 1.0);
    let p1 = Vec4::new(pos.x + 1.0, pos.y, pos.z + 1.0, 1.0);
    let p2 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0, 1.0);
    let p3 = Vec4::new(pos.x, pos.y + 1.0, pos.z + 1.0, 1.0);
    push_quad(vertices, p0, p1, p2, p3, color);
}

fn add_face_back(vertices: &mut Vec<Vertex>, pos: Vec3, color: Vec4) {
    let p0 = Vec4::new(pos.x + 1.0, pos.y, pos.z, 1.0);
    let p1 = Vec4::new(pos.x, pos.y, pos.z, 1.0);
    let p2 = Vec4::new(pos.x, pos.y + 1.0, pos.z, 1.0);
    let p3 = Vec4::new(pos.x + 1.0, pos.y + 1.0, pos.z, 1.0);
    push_quad(vertices, p0, p1, p2, p3, color);
}
