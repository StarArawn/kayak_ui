use crate::KeyCode;

pub enum InputEvent {
    MouseMoved((f32, f32)),
    MouseLeftClick,
    CharEvent { c: char },
    Keyboard { key: KeyCode },
}
