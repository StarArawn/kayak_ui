pub use morphorm::{LayoutType, PositionType, Units};

use crate::{color::Color, render_command::RenderCommand};

#[derive(Debug, Clone, PartialEq)]
pub enum StyleProp<T: Default + Clone> {
    Default,
    Inherit,
    Value(T),
}

impl<T> Default for StyleProp<T>
where
    T: Default + Clone,
{
    fn default() -> Self {
        Self::Default
    }
}

impl<T> StyleProp<T>
where
    T: Default + Clone,
{
    pub fn resolve(&self) -> T {
        match self {
            StyleProp::Default => T::default(),
            StyleProp::Value(value) => value.clone(),
            StyleProp::Inherit => panic!("All styles should be merged before resolving!"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub background_color: StyleProp<Color>,
    pub bottom: StyleProp<Units>,
    pub color: StyleProp<Color>,
    pub height: StyleProp<Units>,
    pub layout_type: StyleProp<LayoutType>,
    pub left: StyleProp<Units>,
    pub position_type: StyleProp<PositionType>,
    pub render_command: StyleProp<RenderCommand>,
    pub right: StyleProp<Units>,
    pub top: StyleProp<Units>,
    pub width: StyleProp<Units>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background_color: StyleProp::Value(Color::TRANSPARENT),
            render_command: StyleProp::Value(RenderCommand::Empty),
            bottom: StyleProp::Default,
            color: StyleProp::Inherit,
            height: StyleProp::Default,
            layout_type: StyleProp::Default,
            left: StyleProp::Default,
            position_type: StyleProp::Default,
            right: StyleProp::Default,
            top: StyleProp::Default,
            width: StyleProp::Default,
        }
    }
}

impl Style {
    pub fn merge(&mut self, other: &Self) {
        match self.background_color {
            StyleProp::Inherit => {
                self.background_color = other.background_color.clone();
            }
            _ => (),
        }
        match self.bottom {
            StyleProp::Inherit => {
                self.bottom = other.bottom.clone();
            }
            _ => (),
        }
        match self.color {
            StyleProp::Inherit => {
                self.color = other.color.clone();
            }
            _ => (),
        }
        match self.height {
            StyleProp::Inherit => {
                self.height = other.height.clone();
            }
            _ => (),
        }
        match self.layout_type {
            StyleProp::Inherit => {
                self.layout_type = other.layout_type.clone();
            }
            _ => (),
        }
        match self.left {
            StyleProp::Inherit => {
                self.left = other.left.clone();
            }
            _ => (),
        }
        match self.position_type {
            StyleProp::Inherit => {
                self.position_type = other.position_type.clone();
            }
            _ => (),
        }
        match self.render_command {
            StyleProp::Inherit => {
                self.render_command = other.render_command.clone();
            }
            _ => (),
        }
        match self.right {
            StyleProp::Inherit => {
                self.right = other.right.clone();
            }
            _ => (),
        }
        match self.top {
            StyleProp::Inherit => {
                self.top = other.top.clone();
            }
            _ => (),
        }
        match self.width {
            StyleProp::Inherit => {
                self.width = other.width.clone();
            }
            _ => (),
        }
    }
}
