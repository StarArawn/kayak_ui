pub use morphorm::{LayoutType, PositionType, Units};

use crate::cursor::PointerEvents;
use crate::{color::Color, render_command::RenderCommand};

#[derive(Debug, Clone, PartialEq)]
pub enum StyleProp<T: Default + Clone> {
    Unset,
    Default,
    Inherit,
    Value(T),
}

impl<T> Default for StyleProp<T>
    where
        T: Default + Clone,
{
    fn default() -> Self {
        Self::Unset
    }
}

impl<T> StyleProp<T>
    where
        T: Default + Clone,
{
    pub fn resolve(&self) -> T {
        match self {
            StyleProp::Unset => T::default(),
            StyleProp::Default => T::default(),
            StyleProp::Value(value) => value.clone(),
            StyleProp::Inherit => panic!("All styles should be merged before resolving!"),
        }
    }
}

macro_rules! define_styles {
    (
        // #[derive(...)]
        // #[cfg(...)]
        $(#[$attr: meta])*
        // pub struct Styles { ... }
        $vis: vis struct $name: ident {
            // pub field_1: StyleProp<f32>,
            // #[cfg(...)]
            // field_2: StyleProp<Color>,
            $(
                $(#[$field_attr: meta])*
                $field_vis: vis $field: ident : $field_type: ty
            ),*
            // (Optional trailing comma)
            $(,)?
        }
    ) => {
        $(#[$attr])*
        $vis struct $name {
            $(
                $(#[$field_attr])*
                $field_vis $field: $field_type
            ),*
        }

        impl $name {

            /// Returns a `Style` object where all fields are set to [`StyleProp::Default`]
            pub(crate) fn defaulted() -> Self {
                Self {
                    $($field: StyleProp::Default),*
                }
            }

            /// If any field is set to [`StyleProp::Inherit`], its value will be taken from `other`
            pub fn inherit(&mut self, other: &Self) {
                 $(
                     if matches!(self.$field, StyleProp::Inherit) {
                         self.$field = other.$field.clone();
                     }
                 )*
            }

            /// Merges two `Style` objects
            ///
            /// Values from `other` are applied to any field that is marked as [`StyleProp::Unset`]
            pub fn merge(&mut self, other: &Self) {
                 $(
                     if matches!(self.$field, StyleProp::Unset) {
                         self.$field = other.$field.clone();
                     }
                 )*
            }
        }
    };
}

define_styles! {
    #[derive(Debug, Default, Clone, PartialEq)]
    pub struct Style {
        pub background_color : StyleProp<Color>,
        pub border_color: StyleProp<Color>,
        pub border_radius: StyleProp<(f32, f32, f32, f32)>,
        pub border: StyleProp<(f32, f32, f32, f32)>,
        pub bottom: StyleProp<Units>,
        pub color: StyleProp<Color>,
        pub height: StyleProp<Units>,
        pub layout_type: StyleProp<LayoutType>,
        pub left: StyleProp<Units>,
        pub margin_bottom: StyleProp<Units>,
        pub margin_left: StyleProp<Units>,
        pub margin_right: StyleProp<Units>,
        pub margin_top: StyleProp<Units>,
        pub max_height: StyleProp<Units>,
        pub max_width: StyleProp<Units>,
        pub min_height: StyleProp<Units>,
        pub min_width: StyleProp<Units>,
        pub padding_bottom: StyleProp<Units>,
        pub padding_left: StyleProp<Units>,
        pub padding_right: StyleProp<Units>,
        pub padding_top: StyleProp<Units>,
        pub pointer_events: StyleProp<PointerEvents>,
        pub position_type: StyleProp<PositionType>,
        pub render_command: StyleProp<RenderCommand>,
        pub right: StyleProp<Units>,
        pub top: StyleProp<Units>,
        pub width: StyleProp<Units>,
    }
}

impl Style {
    /// Returns a `Style` object where all fields are set to their own initial values
    ///
    /// This is the actual "default" to apply over any field marked as [`StyleProp::Unset`] before
    /// resolving the style.
    pub(crate) fn initial() -> Self {
        Self {
            background_color: StyleProp::Default,
            border: StyleProp::Default,
            border_color: StyleProp::Default,
            border_radius: StyleProp::Default,
            bottom: StyleProp::Default,
            color: StyleProp::Inherit,
            height: StyleProp::Default,
            layout_type: StyleProp::Default,
            left: StyleProp::Default,
            margin_bottom: StyleProp::Default,
            margin_left: StyleProp::Default,
            margin_right: StyleProp::Default,
            margin_top: StyleProp::Default,
            max_height: StyleProp::Default,
            max_width: StyleProp::Default,
            min_height: StyleProp::Default,
            min_width: StyleProp::Default,
            padding_bottom: StyleProp::Default,
            padding_left: StyleProp::Default,
            padding_right: StyleProp::Default,
            padding_top: StyleProp::Default,
            pointer_events: StyleProp::Default,
            position_type: StyleProp::Default,
            render_command: StyleProp::Value(RenderCommand::Empty),
            right: StyleProp::Default,
            top: StyleProp::Default,
            width: StyleProp::Default,
        }
    }
}
