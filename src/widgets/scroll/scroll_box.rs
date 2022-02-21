use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{PositionType, Style, Units},
    use_state, widget, Bound, Children, EventType, MutableBound, OnEvent, ScrollUnit, WidgetProps,
};

use kayak_core::styles::LayoutType;
use kayak_core::Color;

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

    // === States === //
    let (_scroll_offset, set_scroll_offset, ..) = use_state!((0.0, 0.0));
    let (is_ready, set_is_ready, ..) = use_state!(false);

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

    let hide_hori = if hide_horizontal {
        hide_horizontal
    } else {
        !always_show_scrollbar && scrollable_width < f32::EPSILON
    };

    let hide_vert = if hide_vertical {
        hide_vertical
    } else {
        !always_show_scrollbar && scrollable_height < f32::EPSILON
    };

    // === Layout === //
    let id = self.get_id();
    if let Some(layout) = context.get_layout(&id) {
        if scroll.content_width > 0.0 || scroll.content_height > 0.0 {
            let mut old = scroll_ctx.get();
            old.scrollbox_width = layout.width;
            old.scrollbox_height = layout.height;
            old.pad_x = if !hide_hori { hori_thickness } else { 0.0 };
            old.pad_y = if !hide_vert { vert_thickness } else { 0.0 };
            scroll_ctx.set(old);

            if !is_ready && old == scroll {
                set_is_ready(true);
            }
        }
    }

    // === Styles === //
    props.styles = Some(
        Style::default()
            .with_style(Style {
                render_command: RenderCommand::Layout.into(),
                ..Default::default()
            })
            .with_style(&props.styles),
    );

    let hbox_styles = Style::default().with_style(Style {
        render_command: RenderCommand::Layout.into(),
        layout_type: LayoutType::Row.into(),
        ..Default::default()
    });
    let vbox_styles = Style::default().with_style(Style {
        render_command: RenderCommand::Layout.into(),
        layout_type: LayoutType::Column.into(),
        ..Default::default()
    });

    let content_styles = Style::default().with_style(Style {
        position_type: PositionType::SelfDirected.into(),
        top: Units::Pixels(scroll_y).into(),
        left: Units::Pixels(scroll_x).into(),
        height: Units::Percentage(100.0).into(),
        width: Units::Percentage(100.0).into(),
        ..Default::default()
    });

    // === Events === //
    let _set_offset = set_scroll_offset;
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
                    <If condition={!hide_hori}>
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
                <If condition={!hide_vert}>
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
