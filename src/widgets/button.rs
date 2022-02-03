use crate::core::{
    render_command::RenderCommand,
    derivative::Derivative,
    Color, OnEvent, rsx, WidgetProps,
    styles::{Style, StyleProp, Units},
    widget, Children, Fragment,
};

#[derive(WidgetProps, Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
pub struct ButtonProps {
    #[props(Styles)]
    pub styles: Option<Style>,
    #[props(Children)]
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub children: Children,
    #[props(OnEvent)]
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub on_event: Option<OnEvent>,
    #[props(Focusable)]
    #[derivative(Default(value = "Some(true)"), PartialEq = "ignore")]
    pub focusable: Option<bool>,
}

#[widget]
pub fn Button(props: ButtonProps) {
    let base_styles = props.styles.clone().unwrap_or_default();
    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Quad),
        border_radius: StyleProp::Value((5.0, 5.0, 5.0, 5.0)),
        height: if base_styles.height == StyleProp::Default {
            StyleProp::Value(Units::Pixels(45.0))
        } else {
            base_styles.height
        },
        background_color: if matches!(base_styles.background_color, StyleProp::Default) {
            StyleProp::Value(Color::new(0.0781, 0.0898, 0.101, 1.0))
        } else {
            base_styles.background_color
        },
        padding_left: StyleProp::Value(Units::Stretch(1.0)),
        padding_right: StyleProp::Value(Units::Stretch(1.0)),
        ..base_styles
    });
    rsx! {
        <Fragment>
            {children}
        </Fragment>
    }
}
