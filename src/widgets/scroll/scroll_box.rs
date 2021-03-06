use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{PositionType, Style, Units},
    widget, Bound, Children, EventType, MutableBound, OnEvent, ScrollUnit, WidgetProps,
};

use kayak_core::styles::LayoutType;
use kayak_core::{Color, GeometryChanged, OnLayout};

use crate::widgets::{Clip, Element, If};

use super::{ScrollBar, ScrollContent, ScrollContext, ScrollMode};

/// Props used by the [`ScrollBox`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct ScrollBoxProps {
    /// If true, always shows scrollbars even when there's nothing to scroll
    ///
    /// Individual scrollbars can still be hidden via [`hide_horizontal`](Self::hide_horizontal)
    /// and [`hide_vertical`](Self::hide_vertical).
    pub always_show_scrollbar: bool,
    /// If true, disables horizontal scrolling
    pub disable_horizontal: bool,
    /// If true, disables vertical scrolling
    pub disable_vertical: bool,
    /// The scroll mode to use
    pub mode: ScrollMode,
    /// If true, hides the horizontal scrollbar
    pub hide_horizontal: bool,
    /// If true, hides the vertical scrollbar
    pub hide_vertical: bool,
    /// The thickness of the scrollbar
    pub scrollbar_thickness: Option<f32>,
    /// The step to scroll by when `ScrollUnit::Line`
    pub scroll_line: Option<f32>,
    /// The color of the scrollbar thumb
    pub thumb_color: Option<Color>,
    /// The styles of the scrollbar thumb
    pub thumb_styles: Option<Style>,
    /// The color of the scrollbar track
    pub track_color: Option<Color>,
    /// The styles of the scrollbar track
    pub track_styles: Option<Style>,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(Children)]
    pub children: Option<Children>,
    #[prop_field(OnLayout)]
    on_layout: Option<OnLayout>,
}

#[widget]
/// A widget that creates a scrollable area for overflowing content
///
/// # Props
///
/// __Type:__ [`ScrollBoxProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ❌        |
/// | `focusable` | ❌        |
///
pub fn ScrollBox(props: ScrollBoxProps) {
    // === Configuration === //
    let always_show_scrollbar = props.always_show_scrollbar;
    let disable_horizontal = props.disable_horizontal;
    let disable_vertical = props.disable_vertical;
    let hide_horizontal = props.hide_horizontal;
    let hide_vertical = props.hide_vertical;
    let mode = props.mode;
    let scrollbar_thickness = props.scrollbar_thickness.unwrap_or(10.0);
    let scroll_line = props.scroll_line.unwrap_or(16.0);
    let thumb_color = props.thumb_color;
    let thumb_styles = props.thumb_styles.clone();
    let track_color = props.track_color;
    let track_styles = props.track_styles.clone();

    // === Scroll === //
    let scroll_ctx = context.create_provider(ScrollContext {
        mode,
        ..Default::default()
    });
    let scroll: ScrollContext = scroll_ctx.get();
    let scroll_x = scroll.scroll_x();
    let scroll_y = scroll.scroll_y();
    let scrollable_width = scroll.scrollable_width();
    let scrollable_height = scroll.scrollable_height();

    let hori_thickness = scrollbar_thickness;
    let vert_thickness = scrollbar_thickness;

    let hide_horizontal =
        hide_horizontal || !always_show_scrollbar && scrollable_width < f32::EPSILON;
    let hide_vertical = hide_vertical || !always_show_scrollbar && scrollable_height < f32::EPSILON;

    {
        let mut next = scroll_ctx.get();
        next.pad_x = if hide_vertical { 0.0 } else { vert_thickness };
        next.pad_y = if hide_horizontal { 0.0 } else { hori_thickness };

        if next.pad_x != scroll.pad_x || next.pad_y != scroll.pad_y {
            scroll_ctx.set(next);
        }
    }

    // === Layout === //
    let _scroll_ctx = scroll_ctx.clone();
    props.on_layout = Some(OnLayout::new(move |_, evt| {
        if evt
            .flags
            .intersects(GeometryChanged::WIDTH_CHANGED | GeometryChanged::HEIGHT_CHANGED)
        {
            let mut next = _scroll_ctx.get();
            next.scrollbox_width = evt.layout.width;
            next.scrollbox_height = evt.layout.height;
            _scroll_ctx.set(next);
        }
    }));

    // === Styles === //
    props.styles = Some(
        Style::default()
            .with_style(Style {
                render_command: RenderCommand::Layout.into(),
                ..Default::default()
            })
            .with_style(&props.styles)
            .with_style(Style {
                width: Units::Stretch(1.0).into(),
                height: Units::Stretch(1.0).into(),
                ..Default::default()
            }),
    );

    let hbox_styles = Style::default().with_style(Style {
        render_command: RenderCommand::Layout.into(),
        layout_type: LayoutType::Row.into(),
        width: Units::Stretch(1.0).into(),
        ..Default::default()
    });
    let vbox_styles = Style::default().with_style(Style {
        render_command: RenderCommand::Layout.into(),
        layout_type: LayoutType::Column.into(),
        width: Units::Stretch(1.0).into(),
        ..Default::default()
    });

    let content_styles = Style::default().with_style(Style {
        position_type: PositionType::SelfDirected.into(),
        top: Units::Pixels(scroll_y).into(),
        left: Units::Pixels(scroll_x).into(),
        ..Default::default()
    });

    // === Events === //
    let event_handler = OnEvent::new(move |_, event| match event.event_type {
        EventType::Scroll(evt) => {
            match evt.delta {
                ScrollUnit::Line { x, y } => {
                    let mut old = scroll_ctx.get();
                    if !disable_horizontal {
                        old.set_scroll_x(scroll_x - x * scroll_line);
                    }
                    if !disable_vertical {
                        old.set_scroll_y(scroll_y + y * scroll_line);
                    }
                    scroll_ctx.set(old);
                }
                ScrollUnit::Pixel { x, y } => {
                    let mut old = scroll_ctx.get();
                    if !disable_horizontal {
                        old.set_scroll_x(scroll_x - x);
                    }
                    if !disable_vertical {
                        old.set_scroll_y(scroll_y + y);
                    }
                    scroll_ctx.set(old);
                }
            }
            event.stop_propagation();
        }
        _ => {}
    });

    // === Render === //
    let children = props.get_children();
    rsx! {
        <Element on_event={Some(event_handler)}>
            <Element styles={Some(hbox_styles)}>
                <Element styles={Some(vbox_styles)}>
                    <Clip>
                        <ScrollContent styles={Some(content_styles)}>
                                {children}
                        </ScrollContent>
                    </Clip>
                    <If condition={!hide_horizontal}>
                        <ScrollBar
                            disabled={disable_horizontal}
                            horizontal={true}
                            thickness={hori_thickness}
                            thumb_color={thumb_color}
                            thumb_styles={thumb_styles}
                            track_color={track_color}
                            track_styles={track_styles}
                        />
                    </If>
                </Element>
                <If condition={!hide_vertical}>
                    <ScrollBar
                        disabled={disable_vertical}
                        thickness={hori_thickness}
                        thumb_color={thumb_color}
                        thumb_styles={thumb_styles}
                        track_color={track_color}
                        track_styles={track_styles}
                    />
                </If>
            </Element>
        </Element>
    }
}
