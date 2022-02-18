use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    widget, Children, OnEvent, WidgetProps,
};

/// Props used by the [`Image`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct ImageProps {
    pub handle: u16,
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
/// A widget that renders an image background
///
/// # Props
///
/// __Type:__ [`ImageProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `focusable` | ✅        |
///
pub fn Image(props: ImageProps) {
    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Image {
            handle: props.handle,
        }),
        ..props.styles.clone().unwrap_or_default()
    });

    rsx! {
        <>
            {children}
        </>
    }
}
