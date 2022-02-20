use crate::cursor::{CursorEvent, ScrollEvent};
use crate::{Index, KeyboardEvent};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    /// The node targeted by this event
    pub target: Index,
    /// The current target of this event
    pub current_target: Index,
    /// The type of event
    pub event_type: EventType,
    /// Indicates whether this event should propagate or not
    pub(crate) should_propagate: bool,
    /// Indicates whether the default action of this event (if any) has been prevented
    pub(crate) default_prevented: bool,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            target: Default::default(),
            current_target: Default::default(),
            event_type: EventType::Click(CursorEvent::default()),
            should_propagate: true,
            default_prevented: false,
        }
    }
}

impl Event {
    /// Create a new event
    ///
    /// This is the preferred method for creating an event as it automatically sets up
    /// propagation and other event metadata in a standardized manner
    pub fn new(target: Index, event_type: EventType) -> Self {
        Self {
            target,
            current_target: target,
            event_type,
            should_propagate: event_type.propagates(),
            default_prevented: false,
        }
    }

    /// Returns whether this event is currently set to propagate
    pub fn propagates(&self) -> bool {
        self.should_propagate
    }

    /// If called, prevents this event from propagating up the hierarchy
    pub fn stop_propagation(&mut self) {
        self.should_propagate = false;
    }

    /// Returns whether this event's default action has been prevented or not
    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented
    }

    /// Prevents this event's default action (if any) from being executed
    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }
}

/// The type of event
///
/// __Note:__ This type implements `PartialEq` and `Hash` in a way that only considers the variant itself,
/// not the underlying data. If full comparisons are needed, they should be done with the inner data or
/// with a custom wrapper.
#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Click(CursorEvent),
    Hover(CursorEvent),
    MouseIn(CursorEvent),
    MouseOut(CursorEvent),
    MouseDown(CursorEvent),
    MouseUp(CursorEvent),
    /// An event that occurs when the user scrolls over a widget
    Scroll(ScrollEvent),
    Focus,
    Blur,
    CharInput {
        c: char,
    },
    KeyUp(KeyboardEvent),
    KeyDown(KeyboardEvent),
}

impl Eq for EventType {}

impl PartialEq for EventType {
    fn eq(&self, _other: &Self) -> bool {
        matches!(self, _other)
    }
}

impl std::hash::Hash for EventType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&std::mem::discriminant(self), state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventCategory {
    Mouse,
    Keyboard,
    Focus,
}

impl EventType {
    /// Returns whether this event type should propagate by default
    ///
    /// For more details on what should and shouldn't propagate, check out the
    /// [W3 specifications](https://www.w3.org/TR/uievents/#event-types), upon which this is based.
    pub fn propagates(&self) -> bool {
        match self {
            // Propagates
            Self::Hover(..) => true,
            Self::Click(..) => true,
            Self::MouseDown(..) => true,
            Self::MouseUp(..) => true,
            Self::Scroll(..) => true,
            Self::CharInput { .. } => true,
            Self::KeyUp(..) => true,
            Self::KeyDown(..) => true,
            // Doesn't Propagate
            Self::MouseIn(..) => false,
            Self::MouseOut(..) => false,
            Self::Focus => false,
            Self::Blur => false,
        }
    }

    /// Get the category of this event
    pub fn event_category(&self) -> EventCategory {
        match self {
            // Mouse
            Self::Hover(..) => EventCategory::Mouse,
            Self::Click(..) => EventCategory::Mouse,
            Self::MouseDown(..) => EventCategory::Mouse,
            Self::MouseUp(..) => EventCategory::Mouse,
            Self::MouseIn(..) => EventCategory::Mouse,
            Self::MouseOut(..) => EventCategory::Mouse,
            Self::Scroll(..) => EventCategory::Mouse,
            // Keyboard
            Self::CharInput { .. } => EventCategory::Keyboard,
            Self::KeyUp(..) => EventCategory::Keyboard,
            Self::KeyDown(..) => EventCategory::Keyboard,
            // Focus
            Self::Focus => EventCategory::Focus,
            Self::Blur => EventCategory::Focus,
        }
    }
}
