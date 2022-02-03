use crate::core::{
    render_command::RenderCommand,
    derivative::Derivative,
    OnEvent, rsx, WidgetProps,
    styles::{Style, StyleProp},
    widget, Children, Fragment,
};

#[derive(WidgetProps, Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
pub struct IfProps {
    pub condition: bool,
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
pub fn If(props: IfProps) {
    if props.condition {
        rsx! {
            <>
                {children}
            </>
        }
    }
}
