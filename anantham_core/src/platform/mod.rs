pub mod input;
pub mod window_controls;

pub mod prelude {
    pub use super::input::{AnanthamAction, default_input_map};
    pub use bevy_window::{PrimaryWindow, Window};
}

use bevy_app::prelude::*;
use bevy_input::InputPlugin;
use bevy_window::WindowPlugin;
use bevy_winit::WinitPlugin;
use leafwing_input_manager::prelude::*;

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WindowPlugin::default(), WinitPlugin::default(), InputPlugin));

        app.add_plugins(InputManagerPlugin::<input::AnanthamAction>::default());
        app.init_resource::<ActionState<input::AnanthamAction>>();
        app.insert_resource(input::default_input_map());

        app.add_systems(Update, window_controls::handle_window_state);
    }
}
