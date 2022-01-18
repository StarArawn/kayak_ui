pub use morphorm::{LayoutType, PositionType, Units};

use crate::cursor::PointerEvents;
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
    pub border_radius: StyleProp<(f32, f32, f32, f32)>,
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
    pub padding_left: StyleProp<Units>,
    pub padding_right: StyleProp<Units>,
    pub padding_top: StyleProp<Units>,
    pub padding_bottom: StyleProp<Units>,
    pub margin_left: StyleProp<Units>,
    pub margin_right: StyleProp<Units>,
    pub margin_top: StyleProp<Units>,
    pub margin_bottom: StyleProp<Units>,
    pub min_width: StyleProp<Units>,
    pub min_height: StyleProp<Units>,
    pub max_width: StyleProp<Units>,
    pub max_height: StyleProp<Units>,
    pub pointer_events: StyleProp<PointerEvents>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background_color: StyleProp::Default,
            border_radius: StyleProp::Default,
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
            padding_left: StyleProp::Default,
            padding_right: StyleProp::Default,
            padding_top: StyleProp::Default,
            padding_bottom: StyleProp::Default,
            margin_left: StyleProp::Default,
            margin_right: StyleProp::Default,
            margin_top: StyleProp::Default,
            margin_bottom: StyleProp::Default,
            min_width: StyleProp::Default,
            min_height: StyleProp::Default,
            max_width: StyleProp::Default,
            max_height: StyleProp::Default,
            pointer_events: StyleProp::Default,
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
        match self.border_radius {
            StyleProp::Inherit => {
                self.border_radius = other.border_radius.clone();
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
        match self.padding_left {
            StyleProp::Inherit => self.padding_left = other.padding_left.clone(),
            _ => (),
        }
        match self.padding_right {
            StyleProp::Inherit => self.padding_right = other.padding_right.clone(),
            _ => (),
        }
        match self.padding_top {
            StyleProp::Inherit => self.padding_top = other.padding_top.clone(),
            _ => (),
        }
        match self.padding_bottom {
            StyleProp::Inherit => self.padding_bottom = other.padding_bottom.clone(),
            _ => (),
        }
        match self.margin_left {
            StyleProp::Inherit => self.margin_left = other.margin_left.clone(),
            _ => (),
        }
        match self.margin_right {
            StyleProp::Inherit => self.margin_right = other.margin_right.clone(),
            _ => (),
        }
        match self.margin_top {
            StyleProp::Inherit => self.margin_top = other.margin_top.clone(),
            _ => (),
        }
        match self.margin_bottom {
            StyleProp::Inherit => self.margin_bottom = other.margin_bottom.clone(),
            _ => (),
        }
    }
}
