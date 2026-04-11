use crate::prelude::*;
use glam::{Mat4, Quat, Vec3};

#[derive(Component, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn compute_view_matrix(&self) -> Mat4 {
        let forward = self.rotation * Vec3::NEG_Z;
        let up = self.rotation * Vec3::Y;
        Mat4::look_to_rh(self.translation, self.translation + forward, up)
    }
}
