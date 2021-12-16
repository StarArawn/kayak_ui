use kayak_ui::bevy::WindowSize;
use kayak_ui::core::derivative::*;
use kayak_ui::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp, Units},
    widget, Binding, Bound, Children,
};

use crate::Clip;

#[widget]
pub fn App(children: Children) {
    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Window),
        ..styles.clone().unwrap_or_default()
    });

    #[cfg(feature = "bevy_renderer")]
    {
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
