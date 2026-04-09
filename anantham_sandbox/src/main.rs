use anantham_core::{App, EngineRunner};
use anantham_render::VoxelRenderPlugin;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mut app = App::new();
    app.add_plugin(VoxelRenderPlugin);

    let runner = EngineRunner::new(app);
    runner.run()?;

    Ok(())
}
