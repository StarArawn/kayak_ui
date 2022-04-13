use crate::styles::Edge;

#[derive(Debug, Clone, PartialEq)]
pub enum RenderCommand {
    Empty,
    /// Represents a node that has no renderable object but contributes to the layout.
    Layout,
    Clip,
    Quad,
    Text {
        content: String,
    },
    Image {
        handle: u16,
    },
    NinePatch {
        border: Edge<f32>,
        handle: u16,
    },
}

impl Default for RenderCommand {
    fn default() -> Self {
        Self::Empty
    }
}
