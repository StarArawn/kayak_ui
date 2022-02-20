use crate::KeyCode;

#[derive(Debug, PartialEq)]
pub enum InputEvent {
    MouseMoved((f32, f32)),
    MouseLeftPress,
    MouseLeftRelease,
    /// An event that occurs when the user scrolls
    Scroll {
        dx: f32,
        dy: f32,
        is_line: bool,
    },
    CharEvent {
        c: char,
    },
    Keyboard {
        key: KeyCode,
        is_pressed: bool,
    },
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
            Self::Scroll { .. } => InputEventCategory::Mouse,
            // Keyboard events
            Self::CharEvent { .. } => InputEventCategory::Keyboard,
            Self::Keyboard { .. } => InputEventCategory::Keyboard,
        }
    }
}
