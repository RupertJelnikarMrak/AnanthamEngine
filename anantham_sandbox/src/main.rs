use anantham_core::{App, EngineRunner, LogPlugin};
use anantham_render::VoxelRenderPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    app.add_plugin(LogPlugin);

    app.add_plugin(VoxelRenderPlugin);

    let runner = EngineRunner::new(app);
    runner.run()?;

    Ok(())
}
