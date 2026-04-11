pub mod input;
pub mod plugin;
pub mod render_bridge;
pub mod spatial;
pub mod voxel;

pub mod prelude {
    pub use bevy_app::prelude::*;
    pub use bevy_ecs::prelude::*;
    pub use bevy_tasks::{AsyncComputeTaskPool, Task};

    pub use crate::plugin::VoxelCorePlugin;
    pub use crate::render_bridge::{ExtractSchedule, RenderSchedule};
    pub use crate::spatial::camera::Camera;
    pub use crate::spatial::transform::Transform;
}
