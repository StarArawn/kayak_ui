/// A color in the sRGB color space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component, 0.0 - 1.0
    pub r: f32,
    /// Green component, 0.0 - 1.0
    pub g: f32,
    /// Blue component, 0.0 - 1.0
    pub b: f32,
    /// Transparency, 0.0 - 1.0
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    /// The black color.
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);

    /// The white color.
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);

    pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
    pub const MAGENTA: Color = Color::rgb(1.0, 0.0, 1.0);
    pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);

    /// A color with no opacity.
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}
