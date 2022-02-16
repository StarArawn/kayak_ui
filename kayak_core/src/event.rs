use crate::cursor::CursorEvent;
use crate::{Index, KeyboardEvent};

/// An event type sent to widgets
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
    /// An event that occurs when the user clicks a widget
    Click(CursorEvent),
    /// An event that occurs when the user hovers the cursor over a widget
    Hover(CursorEvent),
    /// An event that occurs when the user moves the cursor into a widget
    MouseIn(CursorEvent),
    /// An event that occurs when the user moves the cursor out of a widget
    MouseOut(CursorEvent),
    /// An event that occurs when the user presses down on the cursor over a widget
    MouseDown(CursorEvent),
    /// An event that occurs when the user releases the cursor over a widget
    MouseUp(CursorEvent),
    /// An event that occurs when a widget receives focus
    Focus,
    /// An event that occurs when a widget loses focus
    Blur,
    /// An event that occurs when the user types in a character within a _focused_ widget
    CharInput { c: char },
    /// An event that occurs when the user releases a key within a _focused_ widget
    KeyUp(KeyboardEvent),
    /// An event that occurs when the user presses a key down within a _focused_ widget
    KeyDown(KeyboardEvent),
}

impl Eq for EventType {}

/// __Note:__ Only checks if the two event types are the same discriminant
impl PartialEq for EventType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// __Note:__ Only uses the discriminant for hashing
impl std::hash::Hash for EventType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&std::mem::discriminant(self), state);
    }
}

/// The various categories an event can belong to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventCategory {
    /// A category for events related to the mouse/cursor
    Mouse,
    /// A category for events related to the keyboard
    Keyboard,
    /// A category for events related to focus
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
