use crate::Index;

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
}
