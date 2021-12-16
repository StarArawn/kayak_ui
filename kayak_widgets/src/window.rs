use kayak_ui::core::{
    color::Color,
    render_command::RenderCommand,
    rsx,
    styles::{PositionType, Style, StyleProp, Units},
    widget, Children, Fragment,
};

use crate::{Background, Clip, Element, Text};

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
        border_radius: StyleProp::Value((5.0, 5.0, 5.0, 5.0)),
        render_command: StyleProp::Value(RenderCommand::Quad),
        position_type: StyleProp::Value(PositionType::SelfDirected),
        left: StyleProp::Value(Units::Pixels(position.0)),
        top: StyleProp::Value(Units::Pixels(position.1)),
        width: StyleProp::Value(Units::Pixels(size.0)),
        height: StyleProp::Value(Units::Pixels(size.1)),
        ..styles.clone().unwrap_or_default()
    });

    let clip_styles = Style {
        padding_left: StyleProp::Value(Units::Pixels(5.0)),
        padding_right: StyleProp::Value(Units::Pixels(5.0)),
        padding_top: StyleProp::Value(Units::Pixels(5.0)),
        padding_bottom: StyleProp::Value(Units::Pixels(5.0)),
        ..Style::default()
    };

    let title_background_styles = Style {
        background_color: StyleProp::Value(Color::new(0.0781, 0.0898, 0.101, 1.0)),
        border_radius: StyleProp::Value((5.0, 0.0, 0.0, 5.0)),
        height: StyleProp::Value(Units::Pixels(24.0)),
        left: StyleProp::Value(Units::Pixels(0.0)),
        right: StyleProp::Value(Units::Pixels(0.0)),
        top: StyleProp::Value(Units::Pixels(0.0)),
        bottom: StyleProp::Value(Units::Pixels(0.0)),
        padding_left: StyleProp::Value(Units::Pixels(5.0)),
        padding_top: StyleProp::Value(Units::Stretch(1.0)),
        padding_bottom: StyleProp::Value(Units::Stretch(1.0)),
        ..Style::default()
    };

    let title_text_styles = Style {
        height: StyleProp::Value(Units::Pixels(25.0)),
        ..Style::default()
    };

    let content_styles = Style {
        padding_left: StyleProp::Value(Units::Stretch(1.0)),
        padding_right: StyleProp::Value(Units::Stretch(1.0)),
        padding_top: StyleProp::Value(Units::Stretch(1.0)),
        padding_bottom: StyleProp::Value(Units::Stretch(1.0)),
        ..Style::default()
    };

    let title = title.clone();
    rsx! {
        <Fragment>
            <Clip styles={Some(clip_styles)}>
                <Background styles={Some(title_background_styles)}>
                    <Text styles={Some(title_text_styles)} size={16.0} content={title} />
                </Background>
                <Element styles={Some(content_styles)}>
                    {children}
                </Element>
            </Clip>
        </Fragment>
    }
}
