pub mod log;
pub mod platform;
pub mod plugin;
pub mod render_bridge;
pub mod spatial;
pub mod voxel;

pub mod ecs_prelude {
    pub use bevy_ecs::prelude::*;

    pub use crate::spatial::prelude::{Camera, Transform};

    pub use crate::render_bridge::components::{
        ChunkMesh, ExtractedChunk, ExtractedMeshes, ExtractedView, Vertex,
    };
}

pub mod plugin_prelude {
    pub use bevy_app::prelude::*;

    pub use crate::render_bridge::{ExtractSchedule, RenderSchedule};
}

pub mod prelude {
    pub use super::ecs_prelude::*;
    pub use super::plugin_prelude::*;

    pub use crate::platform::prelude::{AppWindow, ScreenResolution};
    pub use bevy_input::prelude::*;
    pub use bevy_tasks::{AsyncComputeTaskPool, Task};

    pub use crate::plugin::AnanthamCorePlugin;
}
