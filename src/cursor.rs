use bevy::{reflect::{FromReflect, Reflect} };
use bevy::prelude::MouseButton;

/// Controls how the cursor interacts on a given node
#[derive(Debug, Reflect, FromReflect, Copy, Clone, PartialEq, Eq)]
pub enum PointerEvents {
    /// Allow all pointer events on this node and its children
    All,
    /// Allow pointer events on this node but not on its children
    SelfOnly,
    /// Allow pointer events on this node's children but not on itself
    ChildrenOnly,
    /// Disallow all pointer events on this node and its children
    None,
}

impl Default for PointerEvents {
    fn default() -> Self {
        Self::All
    }
}


#[derive(   Debug, Copy, Clone, PartialEq)]
pub struct CursorEvent {
    pub pressed: bool,
    pub just_pressed: bool,
    pub just_released: bool,
    pub position: (f32, f32),
    pub mouse_button: MouseButton
}


impl Default for CursorEvent {
    fn default() -> Self {
        Self{ 
            pressed: Default::default(),
            just_pressed: Default::default(),
            just_released: Default::default(),
            position: Default::default(),
            mouse_button: MouseButton::Left
         }
    }
}

/// An event created on scroll
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct ScrollEvent {
    /// The amount scrolled
    pub delta: ScrollUnit,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ScrollUnit {
    /// A scroll unit that goes by a "line of text"
    Line { x: f32, y: f32 },
    /// A scroll unit that goes by individual pixels
    Pixel { x: f32, y: f32 },
}

impl Default for ScrollUnit {
    fn default() -> Self {
        ScrollUnit::Pixel { x: 0.0, y: 0.0 }
    }
}
