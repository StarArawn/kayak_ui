use kayak_core::{
    rsx,
    styles::{LayoutType, Style, StyleProp, Units},
    widget, Color, EventType, Handler, OnEvent,
};
use kayak_widgets::{Background, Text};

use super::delete_button::DeleteButton;

#[widget]
pub fn Card(card_id: usize, name: String, on_delete: Handler<usize>) {
    let name = name.clone();
    let background_styles = Style {
        layout_type: StyleProp::Value(LayoutType::Row),
        background_color: StyleProp::Value(Color::new(0.176, 0.196, 0.215, 1.0)),
        height: StyleProp::Value(Units::Pixels(26.0)),
        top: StyleProp::Value(Units::Pixels(10.0)),
        padding_left: StyleProp::Value(Units::Pixels(5.0)),
        padding_right: StyleProp::Value(Units::Pixels(5.0)),
        padding_top: StyleProp::Value(Units::Pixels(5.0)),
        padding_bottom: StyleProp::Value(Units::Pixels(5.0)),
        ..Style::default()
    };

    let on_delete = on_delete.clone();
    let card_id = *card_id;
    let on_event = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click => {
            on_delete.call(card_id);
        }
        _ => (),
    });

    rsx! {
        <Background styles={Some(background_styles)}>
            <Text size={14.0} content={name} />
            <DeleteButton on_event={Some(on_event)} />
        </Background>
    }
}
