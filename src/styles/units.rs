use bevy::reflect::{FromReflect, Reflect};

/// The layout type determines how nodes will be positioned when directed by the parent
#[derive(Default, Debug, FromReflect, Reflect, Clone, Copy, PartialEq)]
pub enum LayoutType {
    /// Stack child elements horizontally
    Row,
    #[default]
    /// Stack child elements vertically
    Column,
    /// Position child elements into specified rows and columns
    Grid,
}

impl From<LayoutType> for morphorm::LayoutType {
    fn from(val: LayoutType) -> Self {
        match val {
            LayoutType::Column => morphorm::LayoutType::Column,
            LayoutType::Row => morphorm::LayoutType::Row,
            LayoutType::Grid => morphorm::LayoutType::Grid,
        }
    }
}

/// The position type determines whether a node will be positioned in-line with its siblings or seperate
#[derive(Default, Debug, Reflect, FromReflect, Clone, Copy, PartialEq)]
pub enum KPositionType {
    /// Node is positioned relative to parent but ignores its siblings
    SelfDirected,
    #[default]
    /// Node is positioned relative to parent and in-line with siblings
    ParentDirected,
}

impl From<KPositionType> for morphorm::PositionType {
    fn from(val: KPositionType) -> Self {
        match val {
            KPositionType::ParentDirected => morphorm::PositionType::ParentDirected,
            KPositionType::SelfDirected => morphorm::PositionType::SelfDirected,
        }
    }
}

/// Units which describe spacing and size
#[derive(Default, Debug, FromReflect, Reflect, Clone, Copy, PartialEq)]
pub enum Units {
    /// A number of pixels
    Pixels(f32),
    /// A percentage of the parent dimension
    Percentage(f32),
    /// A factor of the remaining free space
    Stretch(f32),
    #[default]
    /// Automatically determine the value
    Auto,
}

impl From<Units> for morphorm::Units {
    fn from(val: Units) -> Self {
        match val {
            Units::Pixels(value) => morphorm::Units::Pixels(value),
            Units::Percentage(value) => morphorm::Units::Percentage(value),
            Units::Stretch(value) => morphorm::Units::Stretch(value),
            Units::Auto => morphorm::Units::Auto,
        }
    }
}

impl Units {
    /// Converts the units to an f32 value
    pub fn value_or(&self, parent_value: f32, auto: f32) -> f32 {
        match self {
            Units::Pixels(pixels) => *pixels,
            Units::Percentage(percentage) => (percentage / 100.0) * parent_value,
            Units::Stretch(_) => auto,
            Units::Auto => auto,
        }
    }

    /// Returns true if the value is in pixels
    pub fn is_pixels(&self) -> bool {
        matches!(self, Units::Pixels(_))
    }

    /// Returns true if the value is a percentage
    pub fn is_percentage(&self) -> bool {
        matches!(self, Units::Percentage(_))
    }

    /// Returns true if the value is a stretch factor
    pub fn is_stretch(&self) -> bool {
        matches!(self, Units::Stretch(_))
    }

    /// Returns true if the value is auto
    pub fn is_auto(&self) -> bool {
        matches!(self, Units::Auto)
    }
}
