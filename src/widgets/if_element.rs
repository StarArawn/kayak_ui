use crate::core::{rsx, styles::Style, widget, Children, OnEvent, WidgetProps};

/// Props used by the [`If`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct IfProps {
    /// If true, the children will be rendered, otherwise nothing will be rendered
    pub condition: bool,
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
/// A widget that _conditionally_ renders its children
///
/// # Props
///
/// __Type:__ [`IfProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `focusable` | ✅        |
///
pub fn If(props: IfProps) {
    if props.condition {
        rsx! {
            <>
                {children}
            </>
        }
    }
}
