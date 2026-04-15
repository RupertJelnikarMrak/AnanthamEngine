use bevy_input::prelude::*;
use bevy_reflect::Reflect;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum AnanthamAction {
    #[actionlike(DualAxis)]
    MoveCamera,
    #[actionlike(DualAxis)]
    LookAround,

    Interact,
    ToggleFullscreen,
    ReleaseMouse,
}

pub fn default_input_map() -> InputMap<AnanthamAction> {
    let mut map = InputMap::default();

    map.insert(AnanthamAction::ToggleFullscreen, KeyCode::F11);
    map.insert(AnanthamAction::ReleaseMouse, KeyCode::Escape);
    map.insert(AnanthamAction::Interact, MouseButton::Left);

    map.insert_dual_axis(AnanthamAction::MoveCamera, VirtualDPad::wasd());

    map.insert_dual_axis(AnanthamAction::LookAround, MouseMove::default());

    map
}
