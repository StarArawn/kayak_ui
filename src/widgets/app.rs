use kayak_core::OnLayout;

use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    widget, Children, OnEvent, WidgetProps,
};

use crate::widgets::Clip;

/// Props used by the [`App`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct AppProps {
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
/// The most common root widget
///
/// # Props
///
/// __Type:__ [`AppProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `on_layout` | ✅        |
/// | `focusable` | ✅        |
///
/// # Using the `bevy_renderer` feature
///
/// When the `bevy_renderer` feature is enabled, this widget will automatically bind to the window size
/// of the Bevy app. This allows it to update on window resize in order to match the width and height of the window.
pub fn App(props: AppProps) {
    #[cfg(feature = "bevy_renderer")]
    {
        use crate::bevy::WindowSize;
        use crate::core::styles::Units;
        use crate::core::{Binding, Bound};
        let window_size = if let Ok(world) = context.get_global::<bevy::prelude::World>() {
            if let Some(window_size) = world.get_resource::<Binding<WindowSize>>() {
                window_size.clone()
            } else {
                return;
            }
        } else {
            return;
        };

        context.bind(&window_size);
        let window_size = window_size.get();
        props.styles = Some(
            Style::default()
                .with_style(Style {
                    render_command: StyleProp::Value(RenderCommand::Layout),
                    width: StyleProp::Value(Units::Pixels(window_size.0)),
                    height: StyleProp::Value(Units::Pixels(window_size.1)),
                    ..Default::default()
                })
                .with_style(&props.styles),
        );
    }

    rsx! {
        <Clip>
            {children}
        </Clip>
    }
}
