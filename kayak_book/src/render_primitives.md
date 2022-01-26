# Render Commands

Kayak UI represents visual widgets by using a rendering command enum:

```rust
pub enum RenderCommand
    /// A widget that renders nothing.
    Empty,
    /// Represents a node that has no renderable object but contributes to the layout.
    Layout,
    /// A widget that renders a rectangular clip area that clips child widgets. 
    Clip,
    /// A widget that renders a rectangular quad.
    Quad,
    /// A widget that renders a string of text.
    Text {
        content: String,
        font: String,
        line_height: f32,
        parent_size: (f32, f32),
        size: f32,
    },
    /// A widget that renders an image.
    Image {
        handle: u16,
    },
    /// A widget that renders a nine patch image.
    NinePatch {
        border: Space,
        handle: u16,
    },
}
```