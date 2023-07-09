use bevy::reflect::Reflect;

/// Layout information for a renderable glyph.
#[derive(Default, Reflect, Debug, Clone, Copy, PartialEq)]
pub struct GlyphRect {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub content: char,
}
