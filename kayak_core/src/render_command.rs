#[derive(Debug, Clone, PartialEq)]
pub enum RenderCommand {
    Empty,
    Clip,
    Quad,
    Text {
        content: String,
        size: f32,
        font: u16,
    },
}

impl Default for RenderCommand {
    fn default() -> Self {
        Self::Empty
    }
}
