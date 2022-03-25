use kayak_core::OnLayout;

use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    widget, Children, OnEvent, WidgetProps,
};

/// Props used by the [`Element`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct ElementProps {
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
/// The most basic widget, equivalent to `div` from HTML.
///
/// It essentially just sets the [`RenderCommand`] of this widget to [`RenderCommand::Layout`].
///
/// # Props
///
/// __Type:__ [`ElementProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `on_layout` | ✅        |
/// | `focusable` | ✅        |
///
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
