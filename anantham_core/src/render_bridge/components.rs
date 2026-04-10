use bevy_ecs::prelude::Resource;
use glam::Mat4;

#[derive(Resource, Clone, Copy)]
pub struct ExtractedView {
    pub view_projection: Mat4,
}
