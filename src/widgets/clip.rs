use kayak_core::OnLayout;

use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp, Units},
    widget, Children, OnEvent, WidgetProps,
};

/// Props used by the [`Clip`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct ClipProps {
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(Children)]
    pub children: Option<Children>,
    #[prop_field(OnEvent)]
    pub on_event: Option<OnEvent>,
    #[prop_field(OnLayout)]
    pub on_layout: Option<OnLayout>,
}

#[widget]
/// A widget that clips its contents to fit the parent container or its designated
/// [`width`](Style::width) and [`height`](Style::height) styling
///
/// # Props
///
/// __Type:__ [`ClipProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `on_layout` | ✅        |
/// | `focusable` | ❌        |
///
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
