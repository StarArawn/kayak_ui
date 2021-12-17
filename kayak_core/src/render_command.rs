use crate::layout_cache::Space;

#[derive(Debug, Clone, PartialEq)]
pub enum RenderCommand {
    Empty,
    Window,
    /// Represents a node that has no renderable object but contributes to the layout.
    Layout,
    Clip,
    Quad,
    Text {
        content: String,
        size: f32,
        font: u16,
    },
    Image {
        handle: u16,
    },
    NinePatch {
        border: Space,
        handle: u16,
    },
}

impl Default for RenderCommand {
    fn default() -> Self {
        Self::Empty
    }
}
