use bevy::{
    prelude::*,
    window::{WindowCreated, WindowResized},
};

/// Tracks the bevy window size.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq)]
pub struct WindowSize(pub f32, pub f32);

pub fn update_window_size(
    mut window_resized_events: EventReader<WindowResized>,
    mut window_created_events: EventReader<WindowCreated>,
    windows: Query<&Window>,
    mut window_size: ResMut<WindowSize>,
) {
    let mut changed_windows = Vec::new();
    // handle resize events. latest events are handled first because we only want to resize each
    // window once
    for event in window_resized_events
        .iter()
        .collect::<Vec<_>>()
        .iter()
        .rev()
    {
        if changed_windows.contains(&event.window) {
            continue;
        }

        changed_windows.push(event.window);
    }

    // handle resize events. latest events are handled first because we only want to resize each
    // window once
    for event in window_created_events
        .iter()
        .collect::<Vec<_>>()
        .iter()
        .rev()
    {
        if changed_windows.contains(&event.window) {
            continue;
        }

        changed_windows.push(event.window);
    }

    for window_entity in changed_windows {
        if let Ok(window) = windows.get(window_entity) {
            let width = window.width();
            let height = window.height();
            *window_size = WindowSize(width, height);
        }
    }
}
