//! Contains code related to the styling of widgets

use std::ops::Add;

pub use super::units::{KPositionType, LayoutType, Units};
use super::BoxShadow;
use bevy::prelude::Color;
use bevy::prelude::Component;
use bevy::prelude::ReflectComponent;
use bevy::prelude::Vec2;
use bevy::prelude::Vec3;
use bevy::prelude::Vec4;
use bevy::reflect::FromReflect;
use bevy::reflect::Reflect;
use bevy::window::CursorIcon;

use crate::cursor::PointerEvents;

use super::AsRefOption;
pub use super::Corner;
pub use super::Edge;
use super::RenderCommand;

/// Just a wrapper around bevy's CursorIcon so we can define a default.
#[derive(Debug, Reflect, Clone, PartialEq, Eq)]
pub struct KCursorIcon(#[reflect(ignore)] pub CursorIcon);

impl FromReflect for KCursorIcon {
    fn from_reflect(_reflect: &dyn Reflect) -> Option<Self> {
        None
    }
}

impl Default for KCursorIcon {
    fn default() -> Self {
        Self(CursorIcon::Default)
    }
}

/// The base container of all style properties
///
/// The default value for this enum is [`StyleProp::Unset`].
#[derive(Debug, Reflect, FromReflect, Clone, PartialEq, Eq)]
pub enum StyleProp<T: Default + Clone + Reflect + FromReflect> {
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
    T: Default + Clone + Reflect + FromReflect,
{
    fn default() -> Self {
        Self::Unset
    }
}

impl<T> StyleProp<T>
where
    T: Default + Clone + Reflect + FromReflect,
{
    /// Resolves this style property into a concrete value.
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

    /// Returns the concrete value of this style property or the provided default.
    ///
    /// If this style property is not [`StyleProp::Value`], then the provided default
    /// will be returned.
    pub fn resolve_or(&self, default: T) -> T {
        if let Self::Value(value) = self {
            value.clone()
        } else {
            default
        }
    }

    /// Returns the concrete value of this style property or computes it from a closure.
    ///
    /// If this style property is not [`StyleProp::Value`], then the return value will be
    /// computed from the provided closure.
    pub fn resolve_or_else<F: FnOnce() -> T>(&self, f: F) -> T {
        if let Self::Value(value) = self {
            value.clone()
        } else {
            f()
        }
    }

    /// Returns the concrete value of this style property or the default value.
    ///
    /// This is similar to the standard [`resolve`](Self::resolve) method, however, it
    /// will _not_ panic on a [`StyleProp::Inherit`].
    pub fn resolve_or_default(&self) -> T {
        if let Self::Value(value) = self {
            value.clone()
        } else {
            T::default()
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

impl<T: Default + Clone + Reflect + FromReflect> From<T> for StyleProp<T> {
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
            pub fn apply<T: AsRefOption<KStyle>>(&mut self, other: T) {
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
            pub fn with_style<T: AsRefOption<KStyle>>(mut self, other: T) -> Self {
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
    #[derive(Component, Reflect, FromReflect, Debug, Default, Clone, PartialEq)]
    #[reflect(Component)]
    pub struct KStyle {
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
        pub border_radius: StyleProp<Corner<f32>>,
        /// The widths of the borders (in pixels)
        ///
        /// The order is (Top, Right, Bottom, Left).
        ///
        /// Only applies to widgets marked [`RenderCommand::Quad`]
        pub border: StyleProp<Edge<f32>>,
        /// The distance between the bottom edge of this widget and the bottom edge of its containing widget
        pub bottom: StyleProp<Units>,
        /// The text color for this widget
        ///
        /// This property defaults to [`StyleProp::Inherit`] meaning that setting this field to some value will
        /// cause all descendents to receive that value, up to the next set value.
        ///
        /// Only applies to widgets marked [`RenderCommand::Text`]
        pub color: StyleProp<Color>,
        /// The spacing between child widgets along the horizontal axis
        pub col_between: StyleProp<Units>,
        /// The cursor icon to display when hovering this widget
        #[reflect(ignore)]
        pub cursor: StyleProp<KCursorIcon>,
        /// The font name for this widget
        ///
        /// Only applies to [`RenderCommand::Text`]
        pub font: StyleProp<String>,
        /// The font size for this widget, in pixels
        ///
        /// Only applies to [`RenderCommand::Text`]
        pub font_size: StyleProp<f32>,
        /// The height of this widget
        pub height: StyleProp<Units>,
        /// The layout method for children of this widget
        pub layout_type: StyleProp<LayoutType>,
        /// The distance between the left edge of this widget and the left edge of its containing widget
        pub left: StyleProp<Units>,
        /// The line height for this widget, in pixels
        ///
        /// Only applies to [`RenderCommand::Text`]
        pub line_height: StyleProp<f32>,
        /// The maximum height of this widget
        pub max_height: StyleProp<Units>,
        /// The maximum width of this widget
        pub max_width: StyleProp<Units>,
        /// The minimum height of this widget
        pub min_height: StyleProp<Units>,
        /// The minimum width of this widget
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
        pub position_type: StyleProp<KPositionType>,
        /// The render method for this widget
        ///
        /// This controls what actually gets rendered and how it's rendered.
        pub render_command: StyleProp<RenderCommand>,
        /// The distance between the right edge of this widget and the right edge of its containing widget
        pub right: StyleProp<Units>,
        /// The spacing between child widgets along the vertical axis
        pub row_between: StyleProp<Units>,
        /// The distance between the top edge of this widget and the top edge of its containing widget
        pub top: StyleProp<Units>,
        /// The width of this widget
        pub width: StyleProp<Units>,
        /// The z-index relative to it's parent.
        pub z_index: StyleProp<i32>,
        /// The list of rows when using the grid layout
        ///
        /// This is specified in the parent widget and the children have to specify their `row_index`.
        pub grid_rows: StyleProp<Vec<Units>>,
        /// The list of columns when using the grid layout
        ///
        /// This is specified in the parent widget and the children have to specify their `col_index`.
        pub grid_cols: StyleProp<Vec<Units>>,
        /// The row index of this widget when using the grid layout
        ///
        /// This references the `grid_rows` property of the parent widget.
        pub row_index: StyleProp<usize>,
        /// The column index of this widget when using the grid layout
        ///
        /// This references the `grid_cols` property of the parent widget.
        pub col_index: StyleProp<usize>,
        /// The number rows that this widget spans when using the grid layout
        ///
        /// Specified in the child widget.
        pub row_span: StyleProp<usize>,
        /// The number columns that this widget spans when using the grid layout
        ///
        /// Specified in the child widget.
        pub col_span: StyleProp<usize>,
        /// The opacity of the widget and it's children
        ///
        /// This is also known as grouped opacity
        /// WARNING! This splits the widget and it's children into a new render pass. So use it sparingly!!!
        pub opacity: StyleProp<f32>,
        /// Box shadow
        /// Currently only applied to quads
        pub box_shadow: StyleProp<Vec<BoxShadow>>,
    }
}

impl KStyle {
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
            font: StyleProp::Inherit,
            font_size: StyleProp::Inherit,
            height: StyleProp::Default,
            layout_type: StyleProp::Default,
            line_height: StyleProp::Inherit,
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
            render_command: StyleProp::Value(RenderCommand::Layout),
            right: StyleProp::Default,
            row_between: StyleProp::Default,
            top: StyleProp::Default,
            width: StyleProp::Default,
            z_index: StyleProp::Default,
            grid_rows: StyleProp::Default,
            grid_cols: StyleProp::Default,
            row_index: StyleProp::Default,
            col_index: StyleProp::Default,
            row_span: StyleProp::Default,
            col_span: StyleProp::Default,
            opacity: StyleProp::Value(1.0),
            box_shadow: StyleProp::Default,
        }
    }

    pub fn lerp(&self, b: &Self, x: f32) -> Self {
        let mut new_styles = self.clone(); // Default to A styles.

        new_styles.background_color = if let StyleProp::Value(color_a) = self.background_color {
            if let StyleProp::Value(color_b) = b.background_color {
                StyleProp::Value(hsv_lerp(&color_a, &color_b, x))
            } else {
                StyleProp::Value(color_a)
            }
        } else {
            self.background_color.clone()
        };

        new_styles.border = if let StyleProp::Value(border_a) = self.border {
            if let StyleProp::Value(border_b) = b.border {
                StyleProp::Value(Edge::new(
                    lerp(border_a.top, border_b.top, x),
                    lerp(border_a.right, border_b.right, x),
                    lerp(border_a.bottom, border_b.bottom, x),
                    lerp(border_a.left, border_b.left, x),
                ))
            } else {
                StyleProp::Value(border_a)
            }
        } else {
            self.border.clone()
        };

        new_styles.border_color = if let StyleProp::Value(color_a) = self.border_color {
            if let StyleProp::Value(color_b) = b.border_color {
                StyleProp::Value(lerp_lch(color_a, color_b, x))
            } else {
                StyleProp::Value(color_a)
            }
        } else {
            self.border_color.clone()
        };

        new_styles.border_radius = if let StyleProp::Value(border_a) = self.border_radius {
            if let StyleProp::Value(border_b) = b.border_radius {
                StyleProp::Value(Corner::new(
                    lerp(border_a.top_left, border_b.top_left, x),
                    lerp(border_a.top_right, border_b.top_right, x),
                    lerp(border_a.bottom_left, border_b.bottom_left, x),
                    lerp(border_a.bottom_right, border_b.bottom_right, x),
                ))
            } else {
                StyleProp::Value(border_a)
            }
        } else {
            self.border_radius.clone()
        };

        new_styles.bottom = lerp_units(&self.bottom, &b.bottom, x);

        new_styles.color = if let StyleProp::Value(color_a) = self.color {
            if let StyleProp::Value(color_b) = b.color {
                StyleProp::Value(hsv_lerp(&color_a, &color_b, x))
            } else {
                StyleProp::Value(color_a)
            }
        } else {
            self.color.clone()
        };

        new_styles.font_size = lerp_f32(&new_styles.font_size, &b.font_size, x);
        new_styles.height = lerp_units(&self.height, &b.height, x);
        new_styles.line_height = lerp_f32(&new_styles.line_height, &b.line_height, x);
        new_styles.left = lerp_units(&self.left, &b.left, x);
        new_styles.max_height = lerp_units(&self.max_height, &b.max_height, x);
        new_styles.max_width = lerp_units(&self.max_width, &b.max_width, x);
        new_styles.min_height = lerp_units(&self.min_height, &b.min_height, x);
        new_styles.min_width = lerp_units(&self.min_width, &b.min_width, x);

        new_styles.offset = if let StyleProp::Value(edge_a) = self.offset {
            if let StyleProp::Value(edge_b) = b.offset {
                StyleProp::Value(Edge::new(
                    get_value(lerp_units(
                        &StyleProp::Value(edge_a.top),
                        &StyleProp::Value(edge_b.top),
                        x,
                    )),
                    get_value(lerp_units(
                        &StyleProp::Value(edge_a.right),
                        &StyleProp::Value(edge_b.right),
                        x,
                    )),
                    get_value(lerp_units(
                        &StyleProp::Value(edge_a.bottom),
                        &StyleProp::Value(edge_b.bottom),
                        x,
                    )),
                    get_value(lerp_units(
                        &StyleProp::Value(edge_a.left),
                        &StyleProp::Value(edge_b.left),
                        x,
                    )),
                ))
            } else {
                StyleProp::Value(edge_a)
            }
        } else {
            self.offset.clone()
        };

        new_styles.padding = if let StyleProp::Value(edge_a) = self.padding {
            if let StyleProp::Value(edge_b) = b.padding {
                StyleProp::Value(Edge::new(
                    get_value(lerp_units(
                        &StyleProp::Value(edge_a.top),
                        &StyleProp::Value(edge_b.top),
                        x,
                    )),
                    get_value(lerp_units(
                        &StyleProp::Value(edge_a.right),
                        &StyleProp::Value(edge_b.right),
                        x,
                    )),
                    get_value(lerp_units(
                        &StyleProp::Value(edge_a.bottom),
                        &StyleProp::Value(edge_b.bottom),
                        x,
                    )),
                    get_value(lerp_units(
                        &StyleProp::Value(edge_a.left),
                        &StyleProp::Value(edge_b.left),
                        x,
                    )),
                ))
            } else {
                StyleProp::Value(edge_a)
            }
        } else {
            self.padding.clone()
        };

        new_styles.padding_bottom = lerp_units(&self.padding_bottom, &b.padding_bottom, x);
        new_styles.padding_left = lerp_units(&self.padding_left, &b.padding_left, x);
        new_styles.padding_right = lerp_units(&self.padding_right, &b.padding_right, x);
        new_styles.padding_top = lerp_units(&self.padding_top, &b.padding_top, x);
        new_styles.right = lerp_units(&self.right, &b.right, x);
        new_styles.top = lerp_units(&self.top, &b.top, x);
        new_styles.width = lerp_units(&self.width, &b.width, x);
        new_styles.opacity = lerp_f32(&self.opacity, &b.opacity, x);

        new_styles
    }
}

fn get_value<T: Default + Clone + FromReflect>(a: StyleProp<T>) -> T {
    match a {
        StyleProp::Value(v) => v,
        _ => {
            panic!("Unexpected lack of value!");
        }
    }
}

fn lerp_f32(prop_a: &StyleProp<f32>, prop_b: &StyleProp<f32>, x: f32) -> StyleProp<f32> {
    if let StyleProp::Value(f_a) = prop_a {
        if let StyleProp::Value(f_b) = prop_b {
            StyleProp::Value(lerp(*f_a, *f_b, x))
        } else {
            StyleProp::Value(*f_a)
        }
    } else {
        prop_a.clone()
    }
}

fn lerp_units(prop_a: &StyleProp<Units>, prop_b: &StyleProp<Units>, x: f32) -> StyleProp<Units> {
    if let StyleProp::Value(unit_a) = prop_a {
        if let StyleProp::Value(unit_b) = prop_b {
            StyleProp::Value(match (unit_a, unit_b) {
                (Units::Pixels(a), Units::Pixels(b)) => Units::Pixels(lerp(*a, *b, x)),
                (Units::Percentage(a), Units::Percentage(b)) => Units::Percentage(lerp(*a, *b, x)),
                (Units::Stretch(a), Units::Stretch(b)) => Units::Stretch(lerp(*a, *b, x)),
                _ => {
                    bevy::prelude::warn!(
                        "Cannot lerp between non-matching units! Unit_A: {:?}, Unit_B: {:?}",
                        unit_a,
                        unit_b
                    );
                    *unit_a
                }
            })
        } else {
            StyleProp::Value(*unit_a)
        }
    } else {
        prop_a.clone()
    }
}

fn lerp_ang(a: f32, b: f32, x: f32) -> f32 {
    let ang = ((((a - b) % std::f32::consts::TAU) + std::f32::consts::PI * 3.)
        % std::f32::consts::TAU)
        - std::f32::consts::PI;
    return ang * x + b;
}

/// Linear interpolation between two colors in Lch space
fn lerp_lch(a: Color, b: Color, x: f32) -> Color {
    let [a_r, a_g, a_b, a_a] = a.as_lcha_f32();
    let [b_r, b_g, b_b, b_a] = b.as_lcha_f32();

    let hue = lerp_ang(a_b, b_b, x);
    let a_xy = Vec2::new(a_r, a_g);
    let b_xy = Vec2::new(b_r, b_g);
    let xy = b_xy.lerp(a_xy, x);

    let alpha = lerp(a_a, b_a, x);

    return Color::Lcha {
        lightness: xy.x,
        chroma: xy.y,
        hue,
        alpha,
    }
    .as_rgba();
}

fn rgb_to_hsv(from: &Color) -> Vec3 {
    // xyz <-> hsv
    let r = from.r();
    let g = from.g();
    let b = from.b();

    let mut res = Vec3::ZERO;

    let min = r.min(g).min(b);
    let max = r.max(g).max(b);

    // Value
    res.z = max;

    let delta = max - min;
    // calc Saturation
    if max != 0.0 {
        res.y = delta / max;
    } else {
        res.x = -1.0;
        res.y = 0.0;

        return res;
    }

    // calc Hue
    if r == max {
        // between Yellow & Magenta
        res.x = (g - b) / delta;
    } else if g == max {
        // cyan to yellow
        res.x = 2.0 + (b - r) / delta;
    } else {
        // b == max // Megnta to cyan
        res.x = 4.0 + (r - g) / delta;
    }

    res.x = res.x * 60.0; // Convert to degrees
    if res.x < 0.0 {
        res.x += 360.0; // Unwrap angle in case of negative
    }

    res
}

fn hsv_to_rgb(from: &Vec3) -> Color {
    let h = from.x;
    let s = from.y;
    let v = from.z;

    // Calc base values
    let c = s * v;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;

    let mut res = Vec4::new(0.0, 0.0, 0.0, 1.0);

    if h >= 0.0 && h < 60.0 {
        res.x = c;
        res.y = x;
        res.z = 0.0;
    } else if h >= 60.0 && h < 120.0 {
        res.x = x;
        res.y = c;
        res.z = 0.0;
    } else if h >= 120.0 && h < 180.0 {
        res.x = 0.0;
        res.y = c;
        res.z = x;
    } else if h >= 180.0 && h < 240.0 {
        res.x = 0.0;
        res.y = x;
        res.z = c;
    } else if h >= 240.0 && h < 300.0 {
        res.x = x;
        res.y = 0.0;
        res.z = c;
    } else {
        res.x = c;
        res.y = 0.0;
        res.z = x;
    }

    res = res + Vec4::new(m, m, m, 0.0);

    Color::from(res)
}

fn hsv_lerp(from: &Color, to: &Color, amount: f32) -> Color {
    let from_a = from.a();
    let to_a = to.a();
    let from = rgb_to_hsv(from);
    let to = rgb_to_hsv(to);
    let mut res = from.lerp(to, amount);

    if from.x < 0.0 {
        res.x = to.x;
    }
    let mut color = hsv_to_rgb(&res);
    color.set_a(lerp(from_a, to_a, amount).clamp(0.0, 1.0));
    color
}

pub(crate) fn lerp(a: f32, b: f32, x: f32) -> f32 {
    a * (1.0 - x) + b * x
}

impl Add for KStyle {
    type Output = KStyle;

    /// Defines the `+` operator for [`Style`]. This is a convenience wrapper of the `self.with_style()` method and useful for concatenating many small `Style` variables.
    /// Similar to `with_style()` In a `StyleA + StyleB` operation, values from `StyleB` are applied to any field of StyleA that are marked as [`StyleProp::Unset`].
    ///
    /// Note: since the changes are applied only to unset fields, addition is *not* commutative. This means StyleA + StyleB != StyleB + StyleA for most cases.
    fn add(self, other: KStyle) -> KStyle {
        self.with_style(other)
    }
}

#[cfg(test)]
mod tests {
    use super::{Edge, KStyle, StyleProp, Units};

    #[test]
    fn styles_should_equal() {
        let mut a = KStyle::default();
        let mut b = KStyle::default();
        assert_eq!(a, b);

        a.height = StyleProp::Default;
        b.height = StyleProp::Inherit;
        assert_ne!(a, b);
    }

    #[test]
    fn style_should_inherit_property() {
        let border = Edge::new(1.0, 2.0, 3.0, 4.0);

        let parent = KStyle {
            border: StyleProp::Value(border),
            ..Default::default()
        };
        let mut child = KStyle {
            border: StyleProp::Inherit,
            ..Default::default()
        };

        child.inherit(&parent);

        assert_eq!(border, child.border.resolve());
    }

    #[test]
    #[should_panic]
    fn style_should_panic_on_resolve_inherit_property() {
        let style = KStyle {
            color: StyleProp::Inherit,
            ..Default::default()
        };

        let _ = style.color.resolve();
    }

    #[test]
    fn style_should_apply_styles_on_unset_property() {
        let mut base_style = KStyle::default();
        let other_style = KStyle {
            width: StyleProp::Value(Units::Pixels(123.0)),
            ..Default::default()
        };

        assert_ne!(base_style.width, other_style.width);

        base_style.apply(&other_style);

        assert_eq!(base_style.width, other_style.width);
    }

    #[test]
    fn style_should_not_apply_styles_on_non_unset_property() {
        let mut base_style = KStyle {
            width: StyleProp::Default,
            ..Default::default()
        };
        let other_style = KStyle {
            width: StyleProp::Value(Units::Pixels(123.0)),
            ..Default::default()
        };

        assert_ne!(base_style.width, other_style.width);

        base_style.apply(&other_style);

        assert_ne!(base_style.width, other_style.width);
    }

    #[test]
    fn style_should_apply_option_style() {
        let mut base_style = KStyle::default();
        let other_style = Some(KStyle {
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
        let expected = KStyle::default();
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

        let expected = KStyle {
            left: expected_left.clone(),
            width: expected_width.clone(),
            height: expected_height.clone(),
            ..Default::default()
        };

        let style = KStyle::default()
            // Pass ownership
            .with_style(KStyle {
                height: expected_height,
                ..Default::default()
            })
            // Pass ownership of option
            .with_style(Some(KStyle {
                left: expected_left,
                ..Default::default()
            }))
            // Pass reference
            .with_style(&KStyle {
                width: expected_width,
                ..Default::default()
            });

        assert_eq!(expected, style);
    }

    #[test]
    fn styles_should_add() {
        let expected_left = StyleProp::Default;
        let expected_width = StyleProp::Value(Units::Stretch(1.0));
        let expected_height = StyleProp::Inherit;

        let expected = KStyle {
            left: expected_left.clone(),
            width: expected_width.clone(),
            height: expected_height.clone(),
            ..Default::default()
        };

        let style_a = KStyle::default();
        let style_b = KStyle {
            height: expected_height,
            ..Default::default()
        };
        let style_c = KStyle {
            left: expected_left,
            ..Default::default()
        };
        let style_d = KStyle {
            width: expected_width,
            ..Default::default()
        };

        assert_eq!(expected, style_a + style_b + style_c + style_d);
    }

    #[test]
    fn value_should_convert_to_property() {
        let expected_width = Units::Pixels(123.0);
        let expected = StyleProp::Value(expected_width);

        let property: StyleProp<_> = expected_width.into();

        assert_eq!(expected, property);
    }

    #[test]
    fn value_should_resolve_with_given_value() {
        let expected: f32 = 123.0;
        let property = StyleProp::Default;

        assert_eq!(expected, property.resolve_or(expected));
        assert_eq!(expected, property.resolve_or_else(|| expected));
        assert_eq!(f32::default(), property.resolve_or_default());
    }
}
