use crate::ecs::{Commands, Component};
use crate::math::Mat4;

#[derive(Component)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: 70.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl Camera {
    /// Computes the projection matrix for the current window aspect ratio.
    /// Includes the necessary Vulkan Y-axis inversion.
    pub fn compute_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        let mut proj = Mat4::perspective_rh(self.fov, aspect_ratio, self.near, self.far);

        // Vulkan's clip space has an inverted Y coordinate compared to standard OpenGL math.
        // We flip the Y axis here so the world renders right-side up.
        proj.y_axis.y *= -1.0;

        proj
    }
}

pub fn spawn_initial_camera(mut commands: Commands) {
    commands.spawn((
        Camera::default(),
        Transform {
            translation: Vec3::new(16.0, 40.0, 16.0),
            ..Default::default()
        },
    ));
}
