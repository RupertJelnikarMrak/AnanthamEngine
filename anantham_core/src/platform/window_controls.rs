use crate::platform::input::AnanthamAction;
use bevy_ecs::prelude::*;
use bevy_window::{
    CursorGrabMode, CursorOptions, MonitorSelection, PrimaryWindow, Window, WindowMode,
};
use leafwing_input_manager::prelude::ActionState;

pub fn handle_window_state(
    mut window_query: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
    action_state: Res<ActionState<AnanthamAction>>,
) {
    let (mut window, mut cursor_options) = match window_query.single_mut() {
        Ok(q) => q,
        Err(err) => {
            tracing::error!("Error trying to query Window: {}", err);
            return;
        }
    };

    if action_state.just_pressed(&AnanthamAction::ToggleFullscreen) {
        window.mode = match window.mode {
            WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            _ => WindowMode::Windowed,
        };
    }

    if action_state.just_pressed(&AnanthamAction::ReleaseMouse) {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }

    if action_state.just_pressed(&AnanthamAction::Interact) {
        cursor_options.grab_mode = CursorGrabMode::Confined;
        cursor_options.visible = false;
    }
}
