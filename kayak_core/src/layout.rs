use crate::layout_cache::Rect;
use crate::Index;
pub use morphorm::GeometryChanged;

/// A layout data sent to widgets on layout.
/// 
/// Similar and interchangeable with [Rect]
/// ```
/// use kayak_core::layout_cache::Rect;
/// use kayak_core::Layout;
/// 
/// let layout = Layout::default();
/// let rect : Rect = layout.into();
/// let layout : Layout = rect.into();
/// ```
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
    pub z: f32,
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
            z_index: layout.z,
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
            z: rect.z_index,
        }
    }
}

///
/// Struct used for [crate::OnLayout] as layout event data.
///
pub struct LayoutEvent {
    /// Layout of target component
    pub layout: Layout,
    /// Flags denoting the layout change.
    pub flags: GeometryChanged,
    /// The node ID of the element receiving the layout event.
    pub target: Index,
}

impl LayoutEvent {
    pub(crate) fn new(rect: Rect, geometry_change: GeometryChanged, index: Index) -> LayoutEvent {
        LayoutEvent {
            layout: rect.into(),
            flags: geometry_change,
            target: index,
        }
    }
}
