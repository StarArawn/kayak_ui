use crate::KeyCode;

/// Events sent to [`KayakContext`](crate::KayakContext) containing user input data
#[derive(Debug, PartialEq)]
pub enum InputEvent {
    /// An event that occurs when the user moves the mouse
    MouseMoved((f32, f32)),
    /// An event that occurs when the user presses the left mouse button
    MouseLeftPress,
    /// An event that occurs when the user releases the left mouse button
    MouseLeftRelease,
    /// An event that occurs when the user scrolls
    Scroll { dx: f32, dy: f32, is_line: bool },
    /// An event that occurs when the user types in a character
    CharEvent { c: char },
    /// An event that occurs when the user presses or releases a key
    Keyboard { key: KeyCode, is_pressed: bool },
}

/// The various categories an input event can belong to
pub enum InputEventCategory {
    /// A category for events related to the mouse/cursor
    Mouse,
    /// A category for events related to the keyboard
    Keyboard,
    // TODO: Gamepad, etc.
}

impl InputEvent {
    /// Get the category of this input event
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
