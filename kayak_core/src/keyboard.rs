use crate::KeyCode;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct KeyboardModifiers {
    /// True if the one of the Control keys is currently pressed
    pub is_ctrl_pressed: bool,
    /// True if the one of the Shift keys is currently pressed
    pub is_shift_pressed: bool,
    /// True if the one of the Alt (or "Option") keys is currently pressed
    pub is_alt_pressed: bool,
    /// True if the one of the Meta keys is currently pressed
    ///
    /// This is the "Command" ("⌘") key on Mac and "Windows" or "Super" on other systems.
    pub is_meta_pressed: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct KeyboardEvent {
    key: KeyCode,
    modifiers: KeyboardModifiers,
}

impl KeyboardEvent {
    pub fn new(key: KeyCode, modifiers: KeyboardModifiers) -> Self {
        Self { key, modifiers }
    }

    /// Returns this event's affected key
    pub fn key(&self) -> KeyCode {
        self.key
    }

    /// Returns all modifiers for this event's key
    pub fn modifiers(&self) -> KeyboardModifiers {
        self.modifiers
    }

    /// Returns true if the one of the Control keys is currently pressed
    pub fn is_ctrl_pressed(&self) -> bool {
        self.modifiers.is_ctrl_pressed
    }

    /// Returns true if the one of the Shift keys is currently pressed
    pub fn is_shift_pressed(&self) -> bool {
        self.modifiers.is_shift_pressed
    }

    /// Returns true if the one of the Alt (or "Option") keys is currently pressed
    pub fn is_alt_pressed(&self) -> bool {
        self.modifiers.is_alt_pressed
    }

    /// Returns true if the one of the Meta keys is currently pressed
    ///
    /// This is the "Command" ("⌘") key on Mac and "Windows" or "Super" on other systems.
    pub fn is_meta_pressed(&self) -> bool {
        self.modifiers.is_meta_pressed
    }
}
