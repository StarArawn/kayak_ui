use crate::core::{
    render_command::RenderCommand,
    derivative::Derivative,
    OnEvent, rsx, WidgetProps,
    styles::{Style, StyleProp},
    widget, Children, Fragment,
};

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct ElementProps {
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(Children)]

    pub children: Option<Children>,
    #[prop_field(OnEvent)]

    pub on_event: Option<OnEvent>,
    #[prop_field(Focusable)]

    pub focusable: Option<bool>,
}

#[widget]
pub fn Element(props: ElementProps) {
    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Layout),
        ..props.styles.clone().unwrap_or_default()
    });

    rsx! {
        <>
            {children}
        </>
    }
}
