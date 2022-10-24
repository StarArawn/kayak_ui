use bevy::prelude::{Bundle, Color, Commands, Component, Entity, In, Query};
use kayak_ui_macros::rsx;

use crate::{
    context::WidgetName,
    event::{Event, EventType},
    event_dispatcher::EventDispatcherContext,
    on_event::OnEvent,
    prelude::{KChildren, WidgetContext},
    styles::{Corner, Edge, KStyle, PositionType, RenderCommand, Units},
    widget::Widget,
    widget_state::WidgetState,
    widgets::{BackgroundBundle, ClipBundle},
};

use super::{map_range, scroll_context::ScrollContext};

/// Props used by the [`ScrollBar`] widget
#[derive(Component, Default, Debug, PartialEq, Clone)]
pub struct ScrollBarProps {
    /// If true, disables the ability to drag
    pub disabled: bool,
    /// If true, displays a horizontal scrollbar instead of a vertical one
    pub horizontal: bool,
    /// The thickness of the scrollbar in pixels
    pub thickness: f32,
    /// The color of the scrollbar thumb
    pub thumb_color: Option<Color>,
    /// The styles of the scrollbar thumb
    pub thumb_styles: Option<KStyle>,
    /// The color of the scrollbar track
    pub track_color: Option<Color>,
    /// The styles of the scrollbar track
    pub track_styles: Option<KStyle>,
}

impl Widget for ScrollBarProps {}

#[derive(Bundle)]
pub struct ScrollBarBundle {
    pub scrollbar_props: ScrollBarProps,
    pub styles: KStyle,
    pub widget_name: WidgetName,
}

impl Default for ScrollBarBundle {
    fn default() -> Self {
        Self {
            scrollbar_props: Default::default(),
            styles: Default::default(),
            widget_name: ScrollBarProps::default().get_name(),
        }
    }
}

pub fn scroll_bar_render(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<(&ScrollBarProps, &mut KStyle)>,
    context_query: Query<&ScrollContext>,
) -> bool {
    if let Ok((scrollbar, mut styles)) = query.get_mut(entity) {
        if let Some(context_entity) = widget_context.get_context_entity::<ScrollContext>(entity) {
            if let Ok(scroll_context) = context_query.get(context_entity) {
                let scroll_x = scroll_context.scroll_x();
                let scroll_y = scroll_context.scroll_y();
                let content_width = scroll_context.content_width();
                let content_height = scroll_context.content_height();
                let scrollable_width = scroll_context.scrollable_width();
                let scrollable_height = scroll_context.scrollable_height();

                let layout = widget_context.get_layout(entity).unwrap_or_default();

                // === Configuration === //
                // let disabled = scrollbar.disabled;
                let horizontal = scrollbar.horizontal;
                let _thickness = scrollbar.thickness;
                let thickness = scrollbar.thickness;
                let thumb_color = scrollbar
                    .thumb_color
                    .unwrap_or_else(|| Color::rgba(0.2981, 0.3098, 0.321, 0.95));
                let thumb_styles = scrollbar.thumb_styles.clone();
                let track_color = scrollbar
                    .track_color
                    .unwrap_or_else(|| Color::rgba(0.1581, 0.1758, 0.191, 0.15));
                let track_styles = scrollbar.track_styles.clone();
                // The size of the thumb as a percentage
                let thumb_size_percent = (if scrollbar.horizontal {
                    layout.width / (content_width - thickness).max(1.0)
                } else {
                    layout.height / (content_height - thickness).max(1.0)
                })
                .clamp(0.1, 1.0);
                // The size of the thumb in pixels
                let thumb_size_pixels = thumb_size_percent
                    * if scrollbar.horizontal {
                        layout.width
                    } else {
                        layout.height
                    };
                let thumb_extents = thumb_size_pixels / 2.0;
                let percent_scrolled = if scrollbar.horizontal {
                    scroll_context.percent_x()
                } else {
                    scroll_context.percent_y()
                };
                // The thumb's offset as a percentage
                let thumb_offset = map_range(
                    percent_scrolled * 100.0,
                    (0.0, 100.0),
                    (0.0, 100.0 - thumb_size_percent * 100.0),
                );

                // === Styles === //
                *styles = KStyle::default().with_style(KStyle {
                    render_command: RenderCommand::Layout.into(),
                    width: if horizontal {
                        Units::Stretch(1.0)
                    } else {
                        Units::Pixels(thickness)
                    }
                    .into(),
                    height: if horizontal {
                        Units::Pixels(thickness)
                    } else {
                        Units::Stretch(1.0)
                    }
                    .into(),
                    ..Default::default()
                });

                let mut track_style =
                    KStyle::default()
                        .with_style(&track_styles)
                        .with_style(KStyle {
                            background_color: track_color.into(),
                            border_radius: Corner::all(thickness / 2.0).into(),
                            ..Default::default()
                        });

                let mut border_color = thumb_color;
                match &mut border_color {
                    Color::Rgba {
                        red,
                        green,
                        blue,
                        alpha,
                    } => {
                        *alpha = (*alpha - 0.2).max(0.0);
                        *red = (*red + 0.1).min(1.0);
                        *green = (*green + 0.1).min(1.0);
                        *blue = (*blue + 0.1).min(1.0);
                    }
                    _ => {}
                }

                let mut thumb_style = KStyle::default()
                    .with_style(KStyle {
                        position_type: PositionType::SelfDirected.into(),
                        ..Default::default()
                    })
                    .with_style(&thumb_styles)
                    .with_style(KStyle {
                        background_color: thumb_color.into(),
                        border_radius: Corner::all(thickness / 2.0).into(),
                        border: Edge::all(1.0).into(),
                        border_color: border_color.into(),
                        ..Default::default()
                    });

                if scrollbar.horizontal {
                    track_style.apply(KStyle {
                        height: Units::Pixels(thickness).into(),
                        width: Units::Stretch(1.0).into(),
                        ..Default::default()
                    });
                    thumb_style.apply(KStyle {
                        height: Units::Pixels(thickness).into(),
                        width: Units::Percentage(thumb_size_percent * 100.0).into(),
                        top: Units::Pixels(0.0).into(),
                        left: Units::Percentage(-thumb_offset).into(),
                        ..Default::default()
                    });
                } else {
                    track_style.apply(KStyle {
                        width: Units::Pixels(thickness).into(),
                        height: Units::Stretch(1.0).into(),
                        ..Default::default()
                    });
                    thumb_style.apply(KStyle {
                        width: Units::Pixels(thickness).into(),
                        height: Units::Percentage(thumb_size_percent * 100.0).into(),
                        top: Units::Percentage(-thumb_offset).into(),
                        left: Units::Pixels(0.0).into(),
                        ..Default::default()
                    });
                }

                // === Events === //
                let on_event = OnEvent::new(
                    move |In((mut event_dispatcher_context, _, mut event, _entity)): In<(
                        EventDispatcherContext,
                        WidgetState,
                        Event,
                        Entity,
                    )>,
                          mut query: Query<&mut ScrollContext>| {
                        if let Ok(mut scroll_context) = query.get_mut(context_entity) {
                            match event.event_type {
                                EventType::MouseDown(data) => {
                                    // --- Capture Cursor --- //
                                    event_dispatcher_context.capture_cursor(event.current_target);
                                    scroll_context.start_pos = data.position.into();
                                    scroll_context.is_dragging = true;

                                    // --- Calculate Start Offsets --- //
                                    // Steps:
                                    // 1. Get position relative to this widget
                                    // 2. Convert relative pos to percentage [0-1]
                                    // 3. Multiply by desired scrollable dimension
                                    // 4. Map value to range padded by half thumb_size (both sides)
                                    // 5. Update scroll
                                    let offset: (f32, f32) = if horizontal {
                                        // 1.
                                        let mut x = data.position.0 - layout.posx;
                                        // 2.
                                        x /= layout.width;
                                        // 3.
                                        x *= -scrollable_width;
                                        // 4.
                                        x = map_range(
                                            x,
                                            (-scrollable_width, 0.0),
                                            (-scrollable_width - thumb_extents, thumb_extents),
                                        );
                                        // 5.
                                        scroll_context.set_scroll_x(x);

                                        (x, scroll_y)
                                    } else {
                                        // 1.
                                        let mut y = data.position.1 - layout.posy;
                                        // 2.
                                        y /= layout.height;
                                        // 3.
                                        y *= -scrollable_height;
                                        // 4.
                                        y = map_range(
                                            y,
                                            (-scrollable_height, 0.0),
                                            (-scrollable_height - thumb_extents, thumb_extents),
                                        );
                                        // 5.
                                        scroll_context.set_scroll_y(y);

                                        (scroll_x, y)
                                    };
                                    scroll_context.start_offset = offset.into();
                                }
                                EventType::MouseUp(..) => {
                                    // --- Release Cursor --- //
                                    event_dispatcher_context.release_cursor(event.current_target);
                                    scroll_context.is_dragging = false;
                                }
                                EventType::Hover(data) => {
                                    if scroll_context.is_dragging {
                                        // --- Move Thumb --- //
                                        // Positional difference (scaled by thumb size)
                                        let pos_diff = (
                                            (scroll_context.start_pos.x - data.position.0)
                                                / thumb_size_percent,
                                            (scroll_context.start_pos.y - data.position.1)
                                                / thumb_size_percent,
                                        );
                                        let start_offset = scroll_context.start_offset;
                                        if horizontal {
                                            scroll_context
                                                .set_scroll_x(start_offset.x + pos_diff.0);
                                        } else {
                                            scroll_context
                                                .set_scroll_y(start_offset.y + pos_diff.1);
                                        }
                                    }
                                }
                                EventType::Scroll(..) if scroll_context.is_dragging => {
                                    // Prevent scrolling while dragging
                                    // This is a bit of a hack to prevent issues when scrolling while dragging
                                    event.stop_propagation();
                                }
                                _ => {}
                            }
                        }

                        (event_dispatcher_context, event)
                    },
                );

                let parent_id = Some(entity);
                rsx! {
                    <BackgroundBundle on_event={on_event} styles={track_style}>
                        <ClipBundle>
                            <BackgroundBundle styles={thumb_style} />
                        </ClipBundle>
                    </BackgroundBundle>
                }
            }
        }
    }
    true
}
