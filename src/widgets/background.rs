use crate::core::{
    render_command::RenderCommand,
    OnEvent, rsx, WidgetProps,
    styles::{Style, StyleProp},
    widget, Children, Fragment,
};

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct BackgroundProps {
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
