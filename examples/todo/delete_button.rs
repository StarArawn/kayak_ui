use kayak_core::CursorIcon;
use kayak_ui::core::{
    color::Color,
    render_command::RenderCommand,
    rsx,
    styles::{Edge, Style, StyleProp, Units},
    use_state, widget, EventType, OnEvent, WidgetProps,
};

use kayak_ui::widgets::{Background, Text};

#[derive(WidgetProps, Clone, Debug, Default, PartialEq)]
pub struct DeleteButtonProps {
    #[prop_field(Styles)]
    styles: Option<Style>,
    #[prop_field(OnEvent)]
    pub on_event: Option<OnEvent>,
}

#[widget]
pub fn DeleteButton(props: DeleteButtonProps) {
    let (color, set_color, ..) = use_state!(Color::new(0.0781, 0.0898, 0.101, 1.0));

    let base_styles = props.styles.clone().unwrap_or_default();
    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Layout),
        height: StyleProp::Value(Units::Pixels(32.0)),
        width: StyleProp::Value(Units::Pixels(30.0)),
        left: StyleProp::Value(Units::Stretch(1.0)),
        ..base_styles
    });

    let background_styles = Some(Style {
        border_radius: StyleProp::Value(Edge::all(5.0)),
        background_color: StyleProp::Value(color),
        cursor: CursorIcon::Hand.into(),
        padding_left: StyleProp::Value(Units::Pixels(8.0)),
        ..Style::default()
    });

    let text_styles = Some(Style {
        cursor: StyleProp::Inherit,
        ..Style::default()
    });

    let on_event = OnEvent::new(move |_, event| match event.event_type {
        EventType::MouseIn(..) => {
            set_color(Color::new(0.0791, 0.0998, 0.201, 1.0));
        }
        EventType::MouseOut(..) => {
            set_color(Color::new(0.0781, 0.0898, 0.101, 1.0));
        }
        _ => {}
    });

    rsx! {
        <Background styles={background_styles} on_event={Some(on_event)}>
            <Text content={"X".to_string()} size={20.0} styles={text_styles} />
        </Background>
    }
}
