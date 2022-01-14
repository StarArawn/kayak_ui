use crate::layout_cache::Space;

#[derive(Debug, Clone, PartialEq)]
pub enum RenderCommand {
    Empty,
    /// Represents a node that has no renderable object but contributes to the layout.
    Layout,
    Clip,
    Quad,
    Text {
        content: String,
        size: f32,
        font: String,
        parent_size: (f32, f32),
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
