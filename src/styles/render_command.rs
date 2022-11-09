use bevy::prelude::{Handle, Image, Vec2};
use kayak_font::Alignment;

use super::Edge;

#[derive(Debug, Clone, PartialEq)]
pub enum RenderCommand {
    Empty,
    /// Represents a node that has no renderable object but contributes to the layout.
    Layout,
    Clip,
    Quad,
    Text {
        content: String,
        alignment: Alignment,
        word_wrap: bool,
    },
    Image {
        handle: Handle<Image>,
    },
    TextureAtlas {
        position: Vec2,
        size: Vec2,
        handle: Handle<Image>,
    },
    NinePatch {
        border: Edge<f32>,
        handle: Handle<Image>,
    },
}

impl Default for RenderCommand {
    fn default() -> Self {
        Self::Empty
    }
}
