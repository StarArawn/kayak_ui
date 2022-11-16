use bevy::reflect::{FromReflect, Reflect};

/// The layout type determines how nodes will be positioned when directed by the parent
#[derive(Debug, FromReflect, Reflect, Clone, Copy, PartialEq)]
pub enum LayoutType {
    /// Stack child elements horizontally
    Row,
    /// Stack child elements vertically
    Column,
    /// Position child elements into specified rows and columns
    Grid,
}

impl Default for LayoutType {
    fn default() -> Self {
        LayoutType::Column
    }
}

impl Into<morphorm::LayoutType> for LayoutType {
    fn into(self) -> morphorm::LayoutType {
        match self {
            LayoutType::Column => morphorm::LayoutType::Column,
            LayoutType::Row => morphorm::LayoutType::Row,
            LayoutType::Grid => morphorm::LayoutType::Grid,
        }
    }
}

/// The position type determines whether a node will be positioned in-line with its siblings or seperate
#[derive(Debug, Reflect, FromReflect, Clone, Copy, PartialEq)]
pub enum KPositionType {
    /// Node is positioned relative to parent but ignores its siblings
    SelfDirected,
    /// Node is positioned relative to parent and in-line with siblings
    ParentDirected,
}

impl Default for KPositionType {
    fn default() -> Self {
        KPositionType::ParentDirected
    }
}

impl Into<morphorm::PositionType> for KPositionType {
    fn into(self) -> morphorm::PositionType {
        match self {
            Self::ParentDirected => morphorm::PositionType::ParentDirected,
            Self::SelfDirected => morphorm::PositionType::SelfDirected,
        }
    }
}

/// Units which describe spacing and size
#[derive(Debug, FromReflect, Reflect, Clone, Copy, PartialEq)]
pub enum Units {
    /// A number of pixels
    Pixels(f32),
    /// A percentage of the parent dimension
    Percentage(f32),
    /// A factor of the remaining free space
    Stretch(f32),
    /// Automatically determine the value
    Auto,
}

impl Default for Units {
    fn default() -> Self {
        Units::Auto
    }
}

impl Into<morphorm::Units> for Units {
    fn into(self) -> morphorm::Units {
        match self {
            Self::Pixels(value) => morphorm::Units::Pixels(value),
            Self::Percentage(value) => morphorm::Units::Percentage(value),
            Self::Stretch(value) => morphorm::Units::Stretch(value),
            Self::Auto => morphorm::Units::Auto,
        }
    }
}

impl Units {
    /// Converts the units to an f32 value
    pub fn value_or(&self, parent_value: f32, auto: f32) -> f32 {
        match self {
            &Units::Pixels(pixels) => pixels,
            &Units::Percentage(percentage) => (percentage / 100.0) * parent_value,
            &Units::Stretch(_) => auto,
            &Units::Auto => auto,
        }
    }

    /// Returns true if the value is in pixels
    pub fn is_pixels(&self) -> bool {
        match self {
            Units::Pixels(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a percentage
    pub fn is_percentage(&self) -> bool {
        match self {
            Units::Percentage(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a stretch factor
    pub fn is_stretch(&self) -> bool {
        match self {
            Units::Stretch(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is auto
    pub fn is_auto(&self) -> bool {
        match self {
            Units::Auto => true,
            _ => false,
        }
    }
}
