//! Contains code related to the styling of widgets

pub use morphorm::{LayoutType, PositionType, Units};

use crate::cursor::PointerEvents;
use crate::{color::Color, render_command::RenderCommand, CursorIcon};

/// The base container of all style properties
///
/// The default value for this enum is [`StyleProp::Unset`].
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
    /// Resolves this style property into a concrete value
    ///
    /// # Panics
    ///
    /// Panics if the style property has been set to [`StyleProp::Inherit`] since it needs to
    /// first be assigned the value of its parent before being resolved.
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

/// A macro that simply wraps the definition struct of [`Style`], allowing
/// some methods to be automatically defined. Otherwise, there would be a _lot_ of
/// copying and pasting, resulting in fragile code.
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

    /// A struct used to define the look of a widget
    ///
    /// # Example
    ///
    /// All fields are `pub`, so you can simply define your styles like:
    ///
    /// ```
    /// # use kayak_core::styles::{Style, StyleProp, Units};
    /// let style = Style {
    ///   width: StyleProp::Value(Units::Pixels(100.0)),
    ///   // Or using `into()` to convert to `StyleProp`
    ///   // width: Units::Pixels(100.0).into(),
    ///   ..Default::default()
    /// };
    /// ```
    ///
    /// You can also create styles using other styles in a builder-like syntax:
    ///
    /// ```
    /// # use kayak_core::styles::{Style, StyleProp, Units};
    /// let style_a = Style {
    ///   width: Units::Pixels(100.0).into(),
    ///   ..Default::default()
    /// };
    /// let style_b = Style {
    ///   height: Units::Pixels(100.0).into(),
    ///   ..Default::default()
    /// };
    ///
    /// let style = Style::default() // <- Initializes all fields as `StyleProp::Unset`
    ///   // Applied first (sets any `StyleProp::Unset` fields)
    ///   .with_style(&style_a)
    ///   // Applied second (sets any remaining `StyleProp::Unset` fields)
    ///   .with_style(&style_b);
    /// ```
    #[derive(Debug, Default, Clone, PartialEq)]
    pub struct Style {
        /// The background color of this widget
        ///
        /// Only applies to widgets marked [`RenderCommand::Quad`]
        pub background_color : StyleProp<Color>,
        /// The color of the border around this widget
        ///
        /// Currently, this controls all border sides.
        ///
        /// Only applies to widgets marked [`RenderCommand::Quad`]
        pub border_color: StyleProp<Color>,
        /// The radius of the corners (in pixels)
        ///
        /// The order is (Top, Right, Bottom, Left).
        ///
        /// Only applies to widgets marked [`RenderCommand::Quad`] and [`RenderCommand::Image`]
        pub border_radius: StyleProp<(f32, f32, f32, f32)>,
        /// The widths of the borders (in pixels)
        ///
        /// The order is (Top, Right, Bottom, Left).
        ///
        /// Only applies to widgets marked [`RenderCommand::Quad`]
        pub border: StyleProp<(f32, f32, f32, f32)>,
        /// The distance between the bottom edge of this widget and the bottom edge of its containing widget
        pub bottom: StyleProp<Units>,
        /// The text color for this widget
        ///
        /// This property defaults to [`StyleProp::Inherit`] meaning that setting this field to some value will
        /// cause all descendents to receive that value, up to the next set value.
        ///
        /// Only applies to widgets marked [`RenderCommand::Text`]
        pub color: StyleProp<Color>,
        /// The cursor icon to display when hovering this widget
        pub cursor: StyleProp<CursorIcon>,
        /// The height of this widget
        pub height: StyleProp<Units>,
        /// The layout method for children of this widget
        pub layout_type: StyleProp<LayoutType>,
        /// The distance between the left edge of this widget and the left edge of its containing widget
        pub left: StyleProp<Units>,
        // TODO: Remove
        pub margin_bottom: StyleProp<Units>,
        // TODO: Remove
        pub margin_left: StyleProp<Units>,
        // TODO: Remove
        pub margin_right: StyleProp<Units>,
        // TODO: Remove
        pub margin_top: StyleProp<Units>,
        /// The maximum height of this widget
        pub max_height: StyleProp<Units>,
        /// The maximum width of this widget
        pub max_width: StyleProp<Units>,
        /// The minimum height of this widget
        pub min_height: StyleProp<Units>,
        /// The minimum width of this widget
        pub min_width: StyleProp<Units>,
        /// The inner padding between the bottom edge of this widget and its children
        ///
        /// A child with their own `bottom` property set to anything other than `Units::Auto` will
        /// override the padding set by this widget
        pub padding_bottom: StyleProp<Units>,
        /// The inner padding between the left edge of this widget and its children
        ///
        /// A child with their own `left` property set to anything other than `Units::Auto` will
        /// override the padding set by this widget
        pub padding_left: StyleProp<Units>,
        /// The inner padding between the right edge of this widget and its children
        ///
        /// A child with their own `right` property set to anything other than `Units::Auto` will
        /// override the padding set by this widget
        pub padding_right: StyleProp<Units>,
        /// The inner padding between the top edge of this widget and its children
        ///
        /// A child with their own `top` property set to anything other than `Units::Auto` will
        /// override the padding set by this widget
        pub padding_top: StyleProp<Units>,
        /// Controls how the pointer interacts with the widget
        ///
        /// This can be used to block pointer events on itself and/or its children if needed, allowing
        /// the event to "pass through" to widgets below.
        pub pointer_events: StyleProp<PointerEvents>,
        /// The position type of the widget relative to its parent
        pub position_type: StyleProp<PositionType>,
        /// The render method for this widget
        ///
        /// This controls what actually gets rendered and how it's rendered.
        pub render_command: StyleProp<RenderCommand>,
        /// The distance between the right edge of this widget and the right edge of its containing widget
        pub right: StyleProp<Units>,
        /// The distance between the top edge of this widget and the top edge of its containing widget
        pub top: StyleProp<Units>,
        /// The width of this widget
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

/// A trait used to allow reading a value as an `Option<&T>`
pub trait AsRefOption<T> {
    fn as_ref_option(&self) -> Option<&T>;
}

impl AsRefOption<Style> for Style {
    fn as_ref_option(&self) -> Option<&Style> {
        Some(&self)
    }
}

impl AsRefOption<Style> for &Style {
    fn as_ref_option(&self) -> Option<&Style> {
        Some(self)
    }
}

impl AsRefOption<Style> for Option<Style> {
    fn as_ref_option(&self) -> Option<&Style> {
        self.as_ref()
    }
}

impl AsRefOption<Style> for &Option<Style> {
    fn as_ref_option(&self) -> Option<&Style> {
        self.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::{Style, StyleProp, Units};

    #[test]
    fn styles_should_equal() {
        let mut a = Style::default();
        let mut b = Style::default();
        assert_eq!(a, b);

        a.height = StyleProp::Default;
        b.height = StyleProp::Inherit;
        assert_ne!(a, b);
    }

    #[test]
    fn style_should_inherit_property() {
        let border = (1.0, 2.0, 3.0, 4.0);

        let parent = Style {
            border: StyleProp::Value(border),
            ..Default::default()
        };
        let mut child = Style {
            border: StyleProp::Inherit,
            ..Default::default()
        };

        child.inherit(&parent);

        assert_eq!(border, child.border.resolve());
    }

    #[test]
    #[should_panic]
    fn style_should_panic_on_resolve_inherit_property() {
        let style = Style {
            color: StyleProp::Inherit,
            ..Default::default()
        };

        let _ = style.color.resolve();
    }

    #[test]
    fn style_should_apply_styles_on_unset_property() {
        let mut base_style = Style::default();
        let other_style = Style {
            width: StyleProp::Value(Units::Pixels(123.0)),
            ..Default::default()
        };

        assert_ne!(base_style.width, other_style.width);

        base_style.apply(&other_style);

        assert_eq!(base_style.width, other_style.width);
    }

    #[test]
    fn style_should_not_apply_styles_on_non_unset_property() {
        let mut base_style = Style {
            width: StyleProp::Default,
            ..Default::default()
        };
        let other_style = Style {
            width: StyleProp::Value(Units::Pixels(123.0)),
            ..Default::default()
        };

        assert_ne!(base_style.width, other_style.width);

        base_style.apply(&other_style);

        assert_ne!(base_style.width, other_style.width);
    }

    #[test]
    fn style_should_apply_option_style() {
        let mut base_style = Style::default();
        let other_style = Some(Style {
            width: StyleProp::Value(Units::Pixels(123.0)),
            ..Default::default()
        });

        assert_ne!(base_style.width, other_style.as_ref().unwrap().width);

        base_style.apply(&other_style);

        assert_eq!(base_style.width, other_style.as_ref().unwrap().width);

        base_style.apply(None);

        assert_eq!(base_style.width, other_style.as_ref().unwrap().width);
    }

    #[test]
    fn style_should_not_apply_none() {
        let expected = Style::default();
        let mut base_style = expected.clone();

        assert_eq!(expected, base_style);
        base_style.apply(None);
        assert_eq!(expected, base_style);
    }

    #[test]
    fn styles_should_be_buildable() {
        let expected_left = StyleProp::Default;
        let expected_width = StyleProp::Value(Units::Stretch(1.0));
        let expected_height = StyleProp::Inherit;

        let expected = Style {
            left: expected_left.clone(),
            width: expected_width.clone(),
            height: expected_height.clone(),
            ..Default::default()
        };

        let style = Style::default()
            // Pass ownership
            .with_style(Style {
                height: expected_height,
                ..Default::default()
            })
            // Pass ownership of option
            .with_style(Some(Style {
                left: expected_left,
                ..Default::default()
            }))
            // Pass reference
            .with_style(&Style {
                width: expected_width,
                ..Default::default()
            });

        assert_eq!(expected, style);
    }

    #[test]
    fn value_should_convert_to_property() {
        let expected_width = Units::Pixels(123.0);
        let expected = StyleProp::Value(expected_width.clone());

        let property: StyleProp<_> = expected_width.into();

        assert_eq!(expected, property);
    }
}