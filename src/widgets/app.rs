use crate::core::derivative::*;
use crate::core::{
    render_command::RenderCommand,
    OnEvent, rsx, WidgetProps,
    styles::{Style, StyleProp},
    widget, Children,
};

use crate::widgets::Clip;

#[derive(WidgetProps, Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
pub struct AppProps {
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
pub fn App(props: AppProps) {
    #[cfg(feature = "bevy_renderer")]
    {
        use crate::bevy::WindowSize;
        use crate::core::styles::Units;
        use crate::core::{Binding, Bound};
        let window_size = if let Ok(world) = context.get_global_state::<bevy::prelude::World>() {
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
        props.styles = Some(Style {
            render_command: StyleProp::Value(RenderCommand::Layout),
            width: StyleProp::Value(Units::Pixels(window_size.0)),
            height: StyleProp::Value(Units::Pixels(window_size.1)),
            ..props.styles.clone().unwrap_or_default()
        });
    }

    rsx! {
        <Clip>
            {children}
        </Clip>
    }
}
