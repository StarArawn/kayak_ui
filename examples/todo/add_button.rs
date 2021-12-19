use kayak_ui::core::{
    color::Color,
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp, Units},
    use_state, widget, Bound, Children, EventType, MutableBound, OnEvent,
};

use kayak_widgets::{Background, Text};

#[widget]
pub fn AddButton(children: Children, styles: Option<Style>) {
    let (color, set_color) = use_state!(Color::new(0.0781, 0.0898, 0.101, 1.0));

    let base_styles = styles.clone().unwrap_or_default();
    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Layout),
        height: StyleProp::Value(Units::Pixels(32.0)),
        width: StyleProp::Value(Units::Pixels(30.0)),
        ..base_styles
    });

    let background_styles = Some(Style {
        border_radius: StyleProp::Value((5.0, 5.0, 5.0, 5.0)),
        background_color: StyleProp::Value(color),
        padding_left: StyleProp::Value(Units::Pixels(9.0)),
        padding_bottom: StyleProp::Value(Units::Pixels(6.0)),
        ..Style::default()
    });

    let on_event = OnEvent::new(move |_, event| match event.event_type {
        EventType::MouseIn => {
            set_color(Color::new(0.0791, 0.0998, 0.201, 1.0));
        }
        EventType::MouseOut => {
            set_color(Color::new(0.0781, 0.0898, 0.101, 1.0));
        }
        _ => {}
    });

    rsx! {
        <Background styles={background_styles} on_event={Some(on_event)}>
            <Text content={"+".to_string()} size={20.0} />
        </Background>
    }
}
