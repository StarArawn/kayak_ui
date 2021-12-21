use crate::core::derivative::*;
use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    widget, Children,
};

use crate::widgets::Clip;

#[widget]
pub fn App(children: Children) {
    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Layout),
        ..styles.clone().unwrap_or_default()
    });

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
        *styles = Some(Style {
            width: StyleProp::Value(Units::Pixels(window_size.0)),
            height: StyleProp::Value(Units::Pixels(window_size.1)),
            ..styles.clone().unwrap_or_default()
        });
    }

    rsx! {
        <Clip>
            {children}
        </Clip>
    }
}
