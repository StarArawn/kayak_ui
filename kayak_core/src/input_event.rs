use crate::KeyCode;

#[derive(Debug, PartialEq)]
pub enum InputEvent {
    MouseMoved((f32, f32)),
    MouseLeftPress,
    MouseLeftRelease,
    CharEvent { c: char },
    Keyboard { key: KeyCode, is_pressed: bool },
}

pub enum InputEventCategory {
    Mouse,
    Keyboard,
    // TODO: Gamepad, etc.
}

impl InputEvent {
    pub fn category(&self) -> InputEventCategory {
        match self {
            // Mouse events
            Self::MouseMoved(..) => InputEventCategory::Mouse,
            Self::MouseLeftPress => InputEventCategory::Mouse,
            Self::MouseLeftRelease => InputEventCategory::Mouse,
            // Keyboard events
            Self::CharEvent { .. } => InputEventCategory::Keyboard,
            Self::Keyboard { .. } => InputEventCategory::Keyboard,
        }
    }
}
