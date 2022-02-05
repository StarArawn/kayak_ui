use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ElementState},
    math::Vec2,
    prelude::{EventReader, IntoExclusiveSystem, MouseButton, Plugin, Res, World},
    render::color::Color,
    window::{CursorMoved, ReceivedCharacter, WindowCreated, WindowResized, Windows},
};

mod bevy_context;
mod camera;
mod key;
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
            .add_system(process_events.exclusive_system())
            .add_system(update.exclusive_system());
    }
}

pub(crate) fn to_bevy_color(color: &kayak_core::color::Color) -> Color {
    Color::rgba(color.r, color.g, color.b, color.a)
}

pub fn update(world: &mut World) {
    if let Some(bevy_context) = world.remove_resource::<BevyContext>() {
        if let Ok(mut context) = bevy_context.kayak_context.write() {
            context.set_global_state(std::mem::take(world));
            context.render();
            *world = context.take_global_state::<World>().unwrap()
        }
        world.insert_resource(bevy_context);
    }
}

pub fn process_events(world: &mut World) {
    let window_size = if let Some(windows) = world.get_resource::<Windows>() {
        if let Some(window) = windows.get_primary() {
            Vec2::new(window.width(), window.height())
        } else {
            panic!("Couldn't find primary window!");
        }
    } else {
        panic!("Couldn't find primary window!");
    };

    if let Some(bevy_context) = world.remove_resource::<BevyContext>() {
        if let Ok(mut context) = bevy_context.kayak_context.write() {
            let mut input_events = Vec::new();

            context.set_global_state(std::mem::take(world));
            context.query_world::<(
                EventReader<CursorMoved>,
                EventReader<MouseButtonInput>,
                EventReader<ReceivedCharacter>,
                EventReader<KeyboardInput>,
            ), _, _>(
                |(
                    mut cursor_moved_events,
                    mut mouse_button_input_events,
                    mut char_input_events,
                    mut keyboard_input_events,
                )| {
                    if let Some(event) = cursor_moved_events.iter().last() {
                        // Currently, we can only handle a single MouseMoved event at a time so everything but the last needs to be skipped
                        input_events.push(InputEvent::MouseMoved((
                            event.position.x as f32,
                            window_size.y - event.position.y as f32,
                        )));
                    }

                    for event in mouse_button_input_events.iter() {
                        match event.button {
                            MouseButton::Left => {
                                if event.state == ElementState::Pressed {
                                    input_events.push(InputEvent::MouseLeftPress);
                                } else if event.state == ElementState::Released {
                                    input_events.push(InputEvent::MouseLeftRelease);
                                }
                            }
                            _ => {}
                        }
                    }

                    for event in char_input_events.iter() {
                        input_events.push(InputEvent::CharEvent { c: event.char });
                    }

                    for event in keyboard_input_events.iter() {
                        if let Some(key_code) = event.key_code {
                            let kayak_key_code = key::convert_virtual_key_code(key_code);
                            input_events.push(InputEvent::Keyboard {
                                key: kayak_key_code,
                                is_pressed: matches!(event.state, ElementState::Pressed),
                            });
                        }
                    }
                },
            );

            context.process_events(input_events);
            *world = context.take_global_state::<World>().unwrap()
        }

        world.insert_resource(bevy_context);
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
