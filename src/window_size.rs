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
    windows: Res<Windows>,
    mut window_size: ResMut<WindowSize>,
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

    for window_id in changed_window_ids {
        if let Some(window) = windows.get(window_id) {
            let width = window.width();
            let height = window.height();
            *window_size = WindowSize(width, height);
        }
    }
}
