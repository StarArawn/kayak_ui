use std::sync::{Arc, RwLock};
use crate::{Index, KeyCode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    pub target: Index,
    pub event_type: EventType,
    pub(crate) should_propagate: bool,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            target: Default::default(),
            event_type: EventType::Click,
            should_propagate: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    Click,
    Hover,
    MouseIn,
    MouseOut,
    Focus,
    Blur,
    CharInput { c: char },
    KeyboardInput { key: KeyCode },
}

/// An event denoting a change of some type
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeEvent<T> {
    pub value: T,
}

/// A handler struct for a [ChangeEvent].
///
/// ## Example
/// ```rust
/// let handler = OnChange::new(move |event| {
///     let value = event.value;
///     // Do something...
/// });
/// ```
#[derive(Clone)]
pub struct OnChange<T>(pub Arc<RwLock<dyn FnMut(ChangeEvent<T>) + Send + Sync + 'static>>);

impl<T> OnChange<T> {
    pub fn new<F: FnMut(ChangeEvent<T>) + Send + Sync + 'static>(f: F) -> Self {
        Self(Arc::new(RwLock::new(f)))
    }
}

impl<T> PartialEq for OnChange<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> std::fmt::Debug for OnChange<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OnChange").finish()
    }
}