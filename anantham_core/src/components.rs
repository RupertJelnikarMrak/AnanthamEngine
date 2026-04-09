use bevy_ecs::prelude::{Component, Resource};
use glam::{Mat4, Quat, Vec3};

#[derive(Component, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

#[derive(Component)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Resource, Clone, Copy)]
pub struct ExtractedView {
    pub view_projection: Mat4,
}
