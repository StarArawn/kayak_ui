use crate::core::{
    render_command::RenderCommand,
    derivative::Derivative,
    OnEvent, rsx, WidgetProps,
    styles::{Style, StyleProp},
    widget, Children, Fragment,
};

#[derive(WidgetProps, Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
pub struct BackgroundProps {
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
pub fn Background(props: BackgroundProps) {
    if props.styles.is_none() {
        props.styles = Some(Style::default())
    }
    props.styles.as_mut().unwrap().render_command = StyleProp::Value(RenderCommand::Quad);
    rsx! {
        <Fragment>
            {children}
        </Fragment>
    }
}
