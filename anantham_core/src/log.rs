use crate::app::{App, Plugin};
use tracing_subscriber::{EnvFilter, fmt};

pub struct LogPlugin;

impl Plugin for LogPlugin {
    fn build(&self, _app: &mut App) {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,anantham_core=debug,anantham_render=debug"));

        let _ = fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_names(true)
            .try_init();

        tracing::info!("LogPlugin initialized.");
    }
}
