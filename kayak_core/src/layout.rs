use morphorm::GeometryChanged;
use crate::layout_cache::Rect;
use crate::{Index, KayakContextRef};

/// A layout event context sent to widgets
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Layout {
    /// width of the component
    pub width: f32,
    /// height of the component
    pub height: f32,
    /// x-coordinates of the component
    pub x: f32,
    /// y-coordinates of the component
    pub y: f32,
    /// z-coordinates of the component
    pub z_index: f32,
}

impl Layout {
    /// Returns the position as a Kayak position type
    pub fn pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

impl From<Layout> for Rect {
    fn from(layout: Layout) -> Self {
        Rect {
            posx: layout.x,
            posy: layout.y,
            width: layout.width,
            height: layout.height,
            z_index: layout.z_index,
        }
    }
}

impl From<Rect> for Layout {
    fn from(rect: Rect) -> Self {
        Layout {
            width: rect.width,
            height: rect.height,
            x: rect.posx,
            y: rect.posy,
            z_index: rect.z_index
        }
    }
}

///
///
///
pub struct LayoutEvent {
    pub layout: Layout,
    pub flags: GeometryChanged,
    pub target: Index,
}