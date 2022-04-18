use bevy::{math::Vec2, prelude::Component, render::color::Color};
use kayak_font::Alignment;

#[derive(Component)]
pub struct Text {
    pub horz_alignment: Alignment,
    pub content: String,
    pub position: Vec2,
    pub size: Vec2,
    pub font_size: f32,
    pub line_height: f32,
    pub color: Color,
}
