use bevy::{
    input::{mouse::MouseButtonInput, ElementState},
    math::Vec2,
    prelude::{EventReader, IntoExclusiveSystem, MouseButton, Plugin, Res, World},
    render::color::Color,
    window::{CursorMoved, WindowCreated, WindowResized, Windows},
};

mod bevy_context;
mod camera;
mod render;

pub use bevy_context::BevyContext;
pub use camera::*;
use kayak_core::{bind, Binding, InputEvent, MutableBound};
pub use render::unified::font::FontMapping;
pub use render::unified::image::ImageManager;

#[derive(Default)]
pub struct BevyKayakUIPlugin;

impl Plugin for BevyKayakUIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(bind(WindowSize::default()))
            .add_plugin(render::BevyKayakUIRenderPlugin)
            .add_plugin(camera::KayakUICameraPlugin)
            .add_system(update_window_size)
            .add_system(process_events)
            .add_system(update.exclusive_system());
    }
}

pub(crate) fn to_bevy_color(color: &kayak_core::color::Color) -> Color {
    Color::rgba(color.r, color.g, color.b, color.a)
}

pub fn update(world: &mut World) {
    let bevy_context = world.remove_resource::<BevyContext>().unwrap();
    if let Ok(mut context) = bevy_context.kayak_context.write() {
        context.set_global_state(std::mem::take(world));
        context.render();
        *world = context.take_global_state::<World>().unwrap()
    }

    world.insert_resource(bevy_context);
}

pub fn process_events(
    bevy_context: Res<BevyContext>,
    windows: Res<Windows>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
) {
    let window_size = if let Some(window) = windows.get_primary() {
        Vec2::new(window.width(), window.height())
    } else {
        panic!("Couldn't find primary window!");
    };

    if let Ok(mut context) = bevy_context.kayak_context.write() {
        let mut input_events = Vec::new();

        for event in cursor_moved_events.iter() {
            input_events.push(InputEvent::MouseMoved((
                event.position.x as f32,
                window_size.y - event.position.y as f32,
            )));
        }

        for event in mouse_button_input_events.iter() {
            match event.button {
                MouseButton::Left => {
                    if event.state == ElementState::Pressed {
                        input_events.push(InputEvent::MouseLeftClick);
                    }
                }
                _ => {}
            }
        }

        context.process_events(input_events);
    }
}

/// Tracks the bevy window size.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct WindowSize(pub f32, pub f32);

fn update_window_size(
    mut window_resized_events: EventReader<WindowResized>,
    mut window_created_events: EventReader<WindowCreated>,
    windows: Res<Windows>,
    window_size: Res<Binding<WindowSize>>,
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
            window_size.set(WindowSize(width, height));
        }
    }
}
