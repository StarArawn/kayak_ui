use bevy::prelude::{BuildChildren, Bundle, Commands, Component, Entity, In, Query, Vec2};

use crate::{
    children::KChildren,
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle},
    widget::Widget,
};

/// Context data provided by a [`ScrollBox`](crate::ScrollBox) widget
#[derive(Component, Default, Debug, Copy, Clone, PartialEq)]
pub struct ScrollContext {
    pub(super) scroll_x: f32,
    pub(super) scroll_y: f32,
    pub(super) content_width: f32,
    pub(super) content_height: f32,
    pub(super) scrollbox_width: f32,
    pub(super) scrollbox_height: f32,
    pub(super) pad_x: f32,
    pub(super) pad_y: f32,
    pub(super) mode: ScrollMode,
    pub(super) is_dragging: bool,
    pub(super) start_pos: Vec2,
    pub(super) start_offset: Vec2,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScrollMode {
    /// Clamps the scroll offset to stay within the scroll range
    Clamped,
    /// Allows infinite scrolling
    Infinite,
}

impl Default for ScrollMode {
    fn default() -> Self {
        ScrollMode::Clamped
    }
}

impl ScrollContext {
    /// Get the current x-axis scroll offset
    pub fn scroll_x(&self) -> f32 {
        self.scroll_x
    }

    /// Get the current y-axis scroll offset
    pub fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    /// The width of the content
    pub fn content_width(&self) -> f32 {
        if self.content_width > self.scrollbox_width {
            self.content_width + self.pad_x
        } else {
            self.content_width
        }
    }

    /// The height of the content
    pub fn content_height(&self) -> f32 {
        if self.content_height > self.scrollbox_height {
            self.content_height + self.pad_y
        } else {
            self.content_height
        }
    }

    /// The total amount that can be scrolled along the x-axis
    pub fn scrollable_width(&self) -> f32 {
        (self.content_width() - self.scrollbox_width).max(0.0)
    }

    /// The total amount that can be scrolled along the y-axis
    pub fn scrollable_height(&self) -> f32 {
        (self.content_height() - self.scrollbox_height).max(0.0)
    }

    /// The current scroll mode
    pub fn mode(&self) -> ScrollMode {
        self.mode
    }

    /// Set the scroll offset along the x-axis
    ///
    /// This automatically accounts for the scroll mode
    pub fn set_scroll_x(&mut self, x: f32) {
        let min = -self.scrollable_width();
        self.scroll_x = match self.mode {
            ScrollMode::Clamped => ScrollContext::clamped(x, min, 0.0),
            ScrollMode::Infinite => x,
        }
    }

    /// Set the scroll offset along the y-axis
    ///
    /// This automatically accounts for the scroll mode
    pub fn set_scroll_y(&mut self, y: f32) {
        let min = -self.scrollable_height();
        self.scroll_y = match self.mode {
            ScrollMode::Clamped => ScrollContext::clamped(y, min, 0.0),
            ScrollMode::Infinite => y,
        };
    }

    /// The percent scrolled along the x-axis
    pub fn percent_x(&self) -> f32 {
        let width = self.scrollable_width();
        if width <= f32::EPSILON {
            // Can't divide by zero
            0.0
        } else {
            self.scroll_x / width
        }
    }

    /// The percent scrolled along the y-axis
    pub fn percent_y(&self) -> f32 {
        let height = self.scrollable_height();
        if height <= f32::EPSILON {
            // Can't divide by zero
            0.0
        } else {
            self.scroll_y / height
        }
    }

    /// Clamps a given value between a range
    fn clamped(value: f32, min: f32, max: f32) -> f32 {
        value.clamp(min, max)
    }
}

#[derive(Component, Default, PartialEq, Clone)]
pub struct ScrollContextProvider {
    initial_value: ScrollContext,
}

impl Widget for ScrollContextProvider {}

#[derive(Bundle)]
pub struct ScrollContextProviderBundle {
    pub scroll_context_provider: ScrollContextProvider,
    pub children: KChildren,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub widget_name: WidgetName,
}

impl Default for ScrollContextProviderBundle {
    fn default() -> Self {
        Self {
            scroll_context_provider: Default::default(),
            children: KChildren::default(),
            styles: Default::default(),
            computed_styles: ComputedStyles::default(),
            widget_name: ScrollContextProvider::default().get_name(),
        }
    }
}

pub fn scroll_context_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<(
        &ScrollContextProvider,
        &KChildren,
        &KStyle,
        &mut ComputedStyles,
    )>,
) -> bool {
    if let Ok((context_provider, children, styles, mut computed_styles)) = query.get_mut(entity) {
        let context_entity = commands
            .spawn(context_provider.initial_value)
            .set_parent(entity)
            .id();
        widget_context.set_context_entity::<ScrollContext>(Some(entity), context_entity);
        *computed_styles = styles.clone().into();
        children.process(&widget_context, Some(entity));
    }

    true
}
