use bevy::{math::Vec2, prelude::Component, render2::color::Color};

#[derive(Component)]
pub struct Text {
    pub content: String,
    pub position: Vec2,
    pub size: Vec2,
    pub font_size: f32,
    pub color: Color,
}
