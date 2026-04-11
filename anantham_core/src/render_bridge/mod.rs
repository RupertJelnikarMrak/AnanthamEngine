pub mod components;
pub mod extract;

use crate::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ExtractSchedule;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RenderSchedule;

pub struct RenderBridgePlugin;

impl Plugin for RenderBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(ExtractSchedule);
        app.init_schedule(RenderSchedule);

        app.init_resource::<components::ExtractedMeshes>();
        app.init_resource::<components::ExtractedView>();

        app.add_systems(
            ExtractSchedule,
            (extract::extract_chunk_meshes, extract::extract_camera_view),
        );
    }
}
