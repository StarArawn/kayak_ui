use crate::core::{
    color::Color,
    render_command::RenderCommand,
    rsx,
    styles::{Corner, Edge, PositionType, Style, StyleProp, Units},
    use_state, widget, Children, EventType, OnEvent, WidgetProps,
};
use kayak_core::CursorIcon;

use crate::widgets::{Background, Clip, Element, Text};

/// Props used by the [`Window`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct WindowProps {
    /// If true, allows the window to be draggable by its title bar
    pub draggable: bool,
    /// The position at which to display the window in pixels
    pub position: (f32, f32),
    /// The size of the window in pixels
    pub size: (f32, f32),
    /// The text to display in the window's title bar
    pub title: String,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(Children)]
    pub children: Option<Children>,
    #[prop_field(OnEvent)]
    pub on_event: Option<OnEvent>,
    #[prop_field(Focusable)]
    pub focusable: Option<bool>,
}

#[widget]
/// A widget that renders a window-like container element
///
/// # Props
///
/// __Type:__ [`WindowProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `focusable` | ✅        |
///
pub fn Window(props: WindowProps) {
    let WindowProps {
        draggable,
        position,
        size,
        title,
        ..
    } = props.clone();

    let (is_dragging, set_is_dragging, ..) = use_state!(false);
    let (offset, set_offset, ..) = use_state!((0.0, 0.0));
    let (pos, set_pos, ..) = use_state!(position);

    let drag_handler = if draggable {
        Some(OnEvent::new(move |ctx, event| match event.event_type {
            EventType::MouseDown(data) => {
                ctx.capture_cursor(event.current_target);
                set_is_dragging(true);
                set_offset((pos.0 - data.position.0, pos.1 - data.position.1));
            }
            EventType::MouseUp(..) => {
                ctx.release_cursor(event.current_target);
                set_is_dragging(false);
            }
            EventType::Hover(data) => {
                if is_dragging {
                    set_pos((offset.0 + data.position.0, offset.1 + data.position.1));
                }
            }
            _ => {}
        }))
    } else {
        None
    };

    props.styles = Some(Style {
        background_color: StyleProp::Value(Color::new(0.125, 0.125, 0.125, 1.0)),
        border_color: StyleProp::Value(Color::new(0.0781, 0.0898, 0.101, 1.0)),
        border: StyleProp::Value(Edge::all(4.0)),
        border_radius: StyleProp::Value(Corner::all(5.0)),
        render_command: StyleProp::Value(RenderCommand::Quad),
        position_type: StyleProp::Value(PositionType::SelfDirected),
        left: StyleProp::Value(Units::Pixels(pos.0)),
        top: StyleProp::Value(Units::Pixels(pos.1)),
        width: StyleProp::Value(Units::Pixels(size.0)),
        height: StyleProp::Value(Units::Pixels(size.1)),
        max_width: StyleProp::Value(Units::Pixels(size.0)),
        max_height: StyleProp::Value(Units::Pixels(size.1)),
        ..props.styles.clone().unwrap_or_default()
    });

    let clip_styles = Style {
        padding: StyleProp::Value(Edge::all(Units::Pixels(5.0))),
        width: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Stretch(1.0)),
        max_width: StyleProp::Value(Units::Pixels(size.0)),
        max_height: StyleProp::Value(Units::Pixels(size.1)),
        ..Style::default()
    };

    let cursor = if draggable {
        if is_dragging {
            CursorIcon::Grabbing
        } else {
            CursorIcon::Grab
        }
    } else {
        CursorIcon::Default
    };

    let title_background_styles = Style {
        background_color: StyleProp::Value(Color::new(0.0781, 0.0898, 0.101, 1.0)),
        border_radius: StyleProp::Value(Corner::all(5.0)),
        cursor: cursor.into(),
        height: StyleProp::Value(Units::Pixels(24.0)),
        width: StyleProp::Value(Units::Stretch(1.0)),
        left: StyleProp::Value(Units::Pixels(0.0)),
        right: StyleProp::Value(Units::Pixels(0.0)),
        top: StyleProp::Value(Units::Pixels(0.0)),
        bottom: StyleProp::Value(Units::Pixels(0.0)),
        padding_left: StyleProp::Value(Units::Pixels(5.0)),
        ..Style::default()
    };

    let title_text_styles = Style {
        height: StyleProp::Value(Units::Pixels(25.0)),
        cursor: StyleProp::Inherit,
        ..Style::default()
    };

    let content_styles = Style {
        padding: StyleProp::Value(Edge::all(Units::Pixels(10.0))),
        ..Style::default()
    };

    let title = title.clone();
    rsx! {
        <Clip styles={Some(clip_styles)}>
            <Background on_event={drag_handler} styles={Some(title_background_styles)}>
                <Text styles={Some(title_text_styles)} size={16.0} content={title} />
            </Background>
            <Element styles={Some(content_styles)}>
                {children}
            </Element>
        </Clip>
    }
}
