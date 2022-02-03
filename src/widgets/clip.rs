use crate::core::{
    render_command::RenderCommand,
    derivative::Derivative,
    OnEvent, rsx, WidgetProps,
    styles::{Style, StyleProp, Units},
    widget, Children, Fragment,
};

#[derive(WidgetProps, Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
pub struct ClipProps {
    #[props(Styles)]
    pub styles: Option<Style>,
    #[props(Children)]
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub children: Children,
    #[props(OnEvent)]
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub on_event: Option<OnEvent>,
}

#[widget]
pub fn Clip(props: ClipProps) {
    let incoming_styles = props.styles.clone().unwrap_or_default();
    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Clip),
        width: if matches!(incoming_styles.width, StyleProp::Value(..)) {
            incoming_styles.width
        } else {
            StyleProp::Value(Units::Stretch(1.0))
        },
        height: if matches!(incoming_styles.height, StyleProp::Value(..)) {
            incoming_styles.height
        } else {
            StyleProp::Value(Units::Stretch(1.0))
        },
        // min_width: StyleProp::Value(Units::Stretch(1.0)),
        // min_height: StyleProp::Value(Units::Stretch(1.0)),
        ..incoming_styles
    });
    rsx! {
        <>
            {children}
        </>
    }
}
