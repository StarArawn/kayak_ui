use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{PositionType, Style, Units},
    use_state, widget, Bound, EventType, MutableBound, OnEvent, WidgetProps,
};
use kayak_core::layout_cache::Rect;
use kayak_core::styles::{Corner, Edge};
use kayak_core::Color;

use crate::widgets::Background;

use super::{map_range, ScrollContext};

/// Props used by the [`ScrollBar`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
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
    pub thumb_styles: Option<Style>,
    /// The color of the scrollbar track
    pub track_color: Option<Color>,
    /// The styles of the scrollbar track
    pub track_styles: Option<Style>,
    #[prop_field(Styles)]
    styles: Option<Style>,
}

#[widget]
/// A widget that displays the current scroll progress within a [`ScrollBox`](crate::ScrollBox) widget
///
/// This widget consists of two main components:
///
/// ### The Thumb
///
/// This is the actual indicator for scroll progress. It also allows you to control the
/// scroll offset by dragging it around.
///
/// ### The Track
///
/// This is the track along which the thumb moves. Clicking anywhere in the track will move
/// the thumb to the clicked position and update the scroll offset.
///
/// # Props
///
/// __Type:__ [`ScrollBarProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ❌        |
/// | `styles`    | ✅        |
/// | `on_event`  | ❌        |
/// | `focusable` | ❌        |
///
/// # Panics
///
/// This widget will panic when used outside the context of a [`ScrollContext`], which
/// is automatically provided by [`ScrollBox`](crate::ScrollBox).
pub fn ScrollBar(props: ScrollBarProps) {
    // === Scroll === //
    let scroll_ctx = context.create_consumer::<ScrollContext>().unwrap();
    let scroll: ScrollContext = scroll_ctx.get();
    let scroll_x = scroll.scroll_x();
    let scroll_y = scroll.scroll_y();
    let content_width = scroll.content_width();
    let content_height = scroll.content_height();
    let scrollable_width = scroll.scrollable_width();
    let scrollable_height = scroll.scrollable_height();

    // === Layout === //
    let id = self.get_id();
    let layout = if let Some(layout) = context.get_layout(&id) {
        *layout
    } else {
        Rect::default()
    };

    // === Configuration === //
    let disabled = props.disabled;
    let horizontal = props.horizontal;
    let _thickness = props.thickness;
    let thickness = props.thickness;
    let thumb_color = props
        .thumb_color
        .unwrap_or_else(|| Color::new(0.2981, 0.3098, 0.321, 0.95));
    let thumb_styles = props.thumb_styles.clone();
    let track_color = props
        .track_color
        .unwrap_or_else(|| Color::new(0.1581, 0.1758, 0.191, 0.15));
    let track_styles = props.track_styles.clone();
    // The size of the thumb as a percentage
    let thumb_size_percent = (if props.horizontal {
        layout.width / (content_width - thickness).max(1.0)
    } else {
        layout.height / (content_height - thickness).max(1.0)
    })
    .clamp(0.1, 1.0);
    // The size of the thumb in pixels
    let thumb_size_pixels = thumb_size_percent
        * if props.horizontal {
            layout.width
        } else {
            layout.height
        };
    let thumb_extents = thumb_size_pixels / 2.0;
    let percent_scrolled = if props.horizontal {
        scroll.percent_x()
    } else {
        scroll.percent_y()
    };
    // The thumb's offset as a percentage
    let thumb_offset = map_range(
        percent_scrolled * 100.0,
        (0.0, 100.0),
        (0.0, 100.0 - thumb_size_percent * 100.0),
    );

    // === Styles === //
    props.styles = Some(
        Style::default().with_style(Style {
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
        }),
    );

    let mut track_style = Style::default()
        .with_style(&track_styles)
        .with_style(Style {
            background_color: track_color.into(),
            border_radius: Corner::all(thickness / 2.0).into(),
            ..Default::default()
        });

    let mut border_color = thumb_color;
    border_color.a = (border_color.a - 0.2).max(0.0);
    border_color.r = (border_color.r + 0.1).min(1.0);
    border_color.g = (border_color.g + 0.1).min(1.0);
    border_color.b = (border_color.b + 0.1).min(1.0);

    let mut thumb_style = Style::default()
        .with_style(Style {
            position_type: PositionType::SelfDirected.into(),
            ..Default::default()
        })
        .with_style(&thumb_styles)
        .with_style(Style {
            background_color: thumb_color.into(),
            border_radius: Corner::all(thickness / 2.0).into(),
            border: Edge::all(1.0).into(),
            border_color: border_color.into(),
            ..Default::default()
        });

    if props.horizontal {
        track_style.apply(Style {
            height: Units::Pixels(thickness).into(),
            width: Units::Stretch(1.0).into(),
            ..Default::default()
        });
        thumb_style.apply(Style {
            height: Units::Pixels(thickness).into(),
            width: Units::Percentage(thumb_size_percent * 100.0).into(),
            top: Units::Pixels(0.0).into(),
            left: Units::Percentage(-thumb_offset).into(),
            ..Default::default()
        });
    } else {
        track_style.apply(Style {
            width: Units::Pixels(thickness).into(),
            height: Units::Stretch(1.0).into(),
            ..Default::default()
        });
        thumb_style.apply(Style {
            width: Units::Pixels(thickness).into(),
            height: Units::Percentage(thumb_size_percent * 100.0).into(),
            top: Units::Percentage(-thumb_offset).into(),
            left: Units::Pixels(0.0).into(),
            ..Default::default()
        });
    }

    // === States === //
    // A state determining whether we are currently dragging the thumb
    let (is_dragging, set_is_dragging, ..) = use_state!(false);
    // A state containing the UI coordinates of the initial click.
    // This is used to get the difference from the current cursor coordinates, so that
    // we can calculate how much the thumb should move.
    let (start_pos, set_start_pos, ..) = use_state!((0.0, 0.0));
    // A state containing the scroll offsets when initially clicked.
    // This is used in conjunction with `start_pos` to calculate the actual scrolled amount.
    let (start_offset, set_start_offset, ..) = use_state!((0.0, 0.0));

    // === Events === //
    let on_track_event = OnEvent::new(move |ctx, event| match event.event_type {
        EventType::MouseDown(data) => {
            // --- Capture Cursor --- //
            ctx.capture_cursor(event.current_target);
            set_start_pos((data.position.0, data.position.1));
            set_is_dragging(true);

            // --- Calculate Start Offsets --- //
            // Steps:
            // 1. Get position relative to this widget
            // 2. Convert relative pos to percentage [0-1]
            // 3. Multiply by desired scrollable dimension
            // 4. Map value to range padded by half thumb_size (both sides)
            // 5. Update scroll
            let mut old = scroll_ctx.get();
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
                old.set_scroll_x(x);

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
                old.set_scroll_y(y);

                (scroll_x, y)
            };

            scroll_ctx.set(old);
            set_start_offset(offset)
        }
        EventType::MouseUp(..) => {
            // --- Release Cursor --- //
            ctx.release_cursor(event.current_target);
            set_is_dragging(false);
        }
        EventType::Hover(data) if is_dragging => {
            // --- Move Thumb --- //
            // Positional difference (scaled by thumb size)
            let pos_diff = (
                (start_pos.0 - data.position.0) / thumb_size_percent,
                (start_pos.1 - data.position.1) / thumb_size_percent,
            );
            let mut old = scroll_ctx.get();
            if horizontal {
                old.set_scroll_x(start_offset.0 + pos_diff.0);
            } else {
                old.set_scroll_y(start_offset.1 + pos_diff.1);
            }
            scroll_ctx.set(old);
        }
        EventType::Scroll(..) if is_dragging => {
            // Prevent scrolling while dragging
            // This is a bit of a hack to prevent issues when scrolling while dragging
            event.stop_propagation();
        }
        _ => {}
    });

    let on_track_event = if disabled { None } else { Some(on_track_event) };

    // === Render === //
    rsx! {
        <Background on_event={on_track_event} styles={Some(track_style)}>
            <Background styles={Some(thumb_style)} />
        </Background>
    }
}
