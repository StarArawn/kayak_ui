mod corner;
mod edge;
mod option_ref;

pub use corner::Corner;
pub use edge::Edge;
pub use morphorm::{LayoutType, PositionType, Units};

use crate::cursor::PointerEvents;
use crate::{color::Color, render_command::RenderCommand, CursorIcon};
use option_ref::AsRefOption;

#[derive(Debug, Clone, PartialEq)]
pub enum StyleProp<T: Default + Clone> {
    /// This prop is unset, meaning its actual value is not determined until style resolution,
    /// wherein it will be set to the property's default value.
    ///
    /// When [applying](Style::apply) styles, only style properties of this type may be
    /// overwritten.
    Unset,
    /// Like [StyleProp::Unset], properties of this type wait until style resolution for their
    /// actual values to be determined, wherein it will be set to the property's default value.
    Default,
    /// Properties of this type inherit their value from their parent (determined at style resolution).
    Inherit,
    /// Set a specific value for this property
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

    /// Returns the first property to not be [unset](StyleProp::Unset)
    ///
    /// If none found, returns [`StyleProp::Unset`]
    pub fn select<'a>(props: &'_ [&'a StyleProp<T>]) -> &'a Self {
        for prop in props {
            if !matches!(prop, StyleProp::Unset) {
                return prop;
            }
        }

        &StyleProp::Unset
    }
}

impl<T: Default + Clone> From<T> for StyleProp<T> {
    fn from(value: T) -> Self {
        StyleProp::Value(value)
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
            ///
            /// This should only be used when default properties are required or desired. Otherwise, you
            /// may be better off using `Style::default` (which sets all properties to [`StyleProp::Unset`]).
            pub fn new_default() -> Self {
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

            /// Applies a `Style` over this one
            ///
            /// Values from `other` are applied to any field in this one that is marked as [`StyleProp::Unset`]
            pub fn apply<T: AsRefOption<Style>>(&mut self, other: T) {
                 if let Some(other) = other.as_ref_option() {
                     $(
                         if matches!(self.$field, StyleProp::Unset) {
                             self.$field = other.$field.clone();
                         }
                     )*
                 }
            }

            /// Applies the given style and returns the updated style
            ///
            /// This is simply a builder-like wrapper around the [`Style::apply`] method.
            pub fn with_style<T: AsRefOption<Style>>(mut self, other: T) -> Self {
                self.apply(other);
                self
            }
        }
    };
}

define_styles! {
    #[derive(Debug, Default, Clone, PartialEq)]
    pub struct Style {
        pub background_color : StyleProp<Color>,
        pub border_color: StyleProp<Color>,
        pub border_radius: StyleProp<Corner<f32>>,
        pub border: StyleProp<Edge<f32>>,
        pub bottom: StyleProp<Units>,
        pub color: StyleProp<Color>,
        /// The spacing between child widgets along the horizontal axis
        pub col_between: StyleProp<Units>,
        pub cursor: StyleProp<CursorIcon>,
        pub height: StyleProp<Units>,
        pub layout_type: StyleProp<LayoutType>,
        pub left: StyleProp<Units>,
        pub max_height: StyleProp<Units>,
        pub max_width: StyleProp<Units>,
        pub min_height: StyleProp<Units>,
        pub min_width: StyleProp<Units>,
        /// The positional offset from this widget's default position
        ///
        /// This property has lower precedence than its more specific counterparts
        /// ([`top`](Self::top), [`right`](Self::right), [`bottom`](Self::bottom), and [`left`](Self::left)),
        /// allowing it to be overridden.
        ///
        /// For widgets with a [`position_type`](Self::position_type) of [`PositionType`](PositionType::ParentDirected)
        /// this acts like margin around the widget. For [`PositionType`](PositionType::SelfDirected) this
        /// acts as the actual position from the parent.
        pub offset: StyleProp<Edge<Units>>,
        /// The inner padding between the edges of this widget and its children
        ///
        /// This property has lower precedence than its more specific counterparts
        /// ([`padding_top`](Self::padding_top), [`padding_right`](Self::padding_right),
        /// [`padding_bottom`](Self::padding_bottom), and [`padding_left`](Self::padding_left)), allowing it
        /// to be overridden.
        ///
        /// A child with their own padding properties set to anything other than [`Units::Auto`] will
        /// override the padding set by this widget.
        pub padding: StyleProp<Edge<Units>>,
        pub padding_bottom: StyleProp<Units>,
        pub padding_left: StyleProp<Units>,
        pub padding_right: StyleProp<Units>,
        pub padding_top: StyleProp<Units>,
        pub pointer_events: StyleProp<PointerEvents>,
        pub position_type: StyleProp<PositionType>,
        pub render_command: StyleProp<RenderCommand>,
        pub right: StyleProp<Units>,
        /// The spacing between child widgets along the vertical axis
        pub row_between: StyleProp<Units>,
        pub top: StyleProp<Units>,
        pub width: StyleProp<Units>,
    }
}

impl Style {
    /// Returns a `Style` object where all fields are set to their own initial values
    ///
    /// This is the actual "default" to apply over any field marked as [`StyleProp::Unset`] before
    /// resolving the style.
    pub fn initial() -> Self {
        Self {
            background_color: StyleProp::Default,
            border: StyleProp::Default,
            border_color: StyleProp::Default,
            border_radius: StyleProp::Default,
            bottom: StyleProp::Default,
            color: StyleProp::Inherit,
            cursor: StyleProp::Inherit,
            col_between: StyleProp::Default,
            height: StyleProp::Default,
            layout_type: StyleProp::Default,
            left: StyleProp::Default,
            max_height: StyleProp::Default,
            max_width: StyleProp::Default,
            min_height: StyleProp::Default,
            min_width: StyleProp::Default,
            offset: StyleProp::Default,
            padding: StyleProp::Default,
            padding_bottom: StyleProp::Default,
            padding_left: StyleProp::Default,
            padding_right: StyleProp::Default,
            padding_top: StyleProp::Default,
            pointer_events: StyleProp::Default,
            position_type: StyleProp::Default,
            render_command: StyleProp::Value(RenderCommand::Empty),
            right: StyleProp::Default,
            row_between: StyleProp::Default,
            top: StyleProp::Default,
            width: StyleProp::Default,
        }
    }
}
