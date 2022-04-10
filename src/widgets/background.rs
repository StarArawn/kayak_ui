use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    widget, Children, Fragment, OnEvent, WidgetProps,
};
use kayak_core::OnLayout;

/// Props used by the [`Background`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct BackgroundProps {
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(Children)]
    pub children: Option<Children>,
    #[prop_field(OnEvent)]
    pub on_event: Option<OnEvent>,
    #[prop_field(OnLayout)]
    pub on_layout: Option<OnLayout>,
    #[prop_field(Focusable)]
    pub focusable: Option<bool>,
}

#[widget]
/// A widget that provides a simple, rectangular background
///
/// # Props
///
/// __Type:__ [`BackgroundProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `on_layout` | ✅        |
/// | `focusable` | ✅        |
///
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
