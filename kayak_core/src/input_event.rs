use crate::KeyCode;

pub enum InputEvent {
    MouseMoved((f32, f32)),
    MouseLeftPress,
    MouseLeftRelease,
    CharEvent { c: char },
    Keyboard { key: KeyCode },
}
