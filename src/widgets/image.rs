use crate::core::{
    render_command::RenderCommand,
    derivative::Derivative,
    OnEvent, rsx, WidgetProps,
    styles::{Style, StyleProp},
    widget, Children, Fragment,
};

#[derive(WidgetProps, Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
pub struct ImageProps {
    pub handle: u16,
    #[props(Styles)]
    pub styles: Option<Style>,
    #[props(Children)]
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub children: Children,
    #[props(OnEvent)]
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub on_event: Option<OnEvent>,
    #[props(Focusable)]
    #[derivative(Default(value = "None"), PartialEq = "ignore")]
    pub focusable: Option<bool>,
}

#[widget]
pub fn Image(props: ImageProps) {
    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Image { handle: props.handle }),
        ..props.styles.clone().unwrap_or_default()
    });

    rsx! {
        <>
            {children}
        </>
    }
}
