pub mod handler;
pub mod window;

pub mod prelude {
    pub use super::window::*;
}

use crate::plugin_prelude::*;
use handler::anantham_winit_runner;

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.set_runner(anantham_winit_runner);
    }
}
