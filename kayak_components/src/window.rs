use kayak_core::{
    color::Color,
    render_command::RenderCommand,
    rsx,
    styles::{PositionType, Style, StyleProp, Units},
    widget, Children, Fragment,
};

use crate::{Background, Clip, Text};

#[widget]
pub fn Window(
    children: Children,
    styles: Option<Style>,
    position: (f32, f32),
    size: (f32, f32),
    title: String,
) {
    *styles = Some(Style {
        background_color: StyleProp::Value(Color::new(0.125, 0.125, 0.125, 1.0)),
        render_command: StyleProp::Value(RenderCommand::Quad),
        position_type: StyleProp::Value(PositionType::SelfDirected),
        left: StyleProp::Value(Units::Pixels(position.0)),
        top: StyleProp::Value(Units::Pixels(position.1)),
        width: StyleProp::Value(Units::Pixels(size.0)),
        height: StyleProp::Value(Units::Pixels(size.1)),
        ..styles.clone().unwrap_or_default()
    });

    let title_background_styles = Style {
        background_color: StyleProp::Value(Color::new(0.0781, 0.0898, 0.101, 1.0)),
        height: StyleProp::Value(Units::Pixels(24.0)),
        ..Style::default()
    };

    let title_text_styles = Style { ..Style::default() };

    let title = title.clone();
    rsx! {
        <Fragment>
            <Clip>
                <Background styles={Some(title_background_styles)}>
                    <Text styles={Some(title_text_styles)} size={14.0} content={title}>{}</Text>
                </Background>
            </Clip>
            <Clip>
                {children}
            </Clip>
        </Fragment>
    }
}
