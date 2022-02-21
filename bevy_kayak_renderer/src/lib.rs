use bevy::{
    prelude::*,
    window::{WindowCreated, WindowResized},
};

pub mod camera;
pub mod render;

pub use camera::*;

#[derive(Default)]
pub struct BevyKayakRendererPlugin;

impl Plugin for BevyKayakRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(update_window_size)
            .init_resource::<WindowSize>()
            .add_plugin(render::BevyKayakUIRenderPlugin)
            .add_plugin(camera::KayakUICameraPlugin);
    }
}

/// Tracks the bevy window size.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct WindowSize(pub f32, pub f32);

fn update_window_size(
    mut window_resized_events: EventReader<WindowResized>,
    mut window_created_events: EventReader<WindowCreated>,
) {
    let mut changed_window_ids = Vec::new();
    // handle resize events. latest events are handled first because we only want to resize each
    // window once
    for event in window_resized_events.iter().rev() {
        if changed_window_ids.contains(&event.id) {
            continue;
        }

        changed_window_ids.push(event.id);
    }

    // handle resize events. latest events are handled first because we only want to resize each
    // window once
    for event in window_created_events.iter().rev() {
        if changed_window_ids.contains(&event.id) {
            continue;
        }

        changed_window_ids.push(event.id);
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Corner<T> {
    pub top_left: T,
    pub top_right: T,
    pub bottom_left: T,
    pub bottom_right: T,
}
