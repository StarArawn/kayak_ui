use bevy::{
    input::{mouse::MouseButtonInput, ElementState},
    math::Vec2,
    prelude::{EventReader, MouseButton, Plugin, Res, ResMut},
    render2::color::Color,
    window::{CursorMoved, Windows},
};

mod bevy_context;
mod camera;
mod render;

pub use bevy_context::BevyContext;
pub use camera::*;
use kayak_core::InputEvent;

#[derive(Default)]
pub struct BevyKayakUIPlugin;

impl Plugin for BevyKayakUIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(render::BevyKayakUIRenderPlugin)
            .add_plugin(camera::KayakUICameraPlugin)
            .add_system(update);
    }
}

pub(crate) fn to_bevy_color(color: &kayak_core::color::Color) -> Color {
    Color::rgba(color.r, color.g, color.b, color.a)
}

pub fn update(
    bevy_context: ResMut<BevyContext>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    windows: Res<Windows>,
) {
    let window_size = if let Some(window) = windows.get_primary() {
        Vec2::new(window.width(), window.height())
    } else {
        panic!("Couldn't find primary window!");
    };

    if let Ok(mut context) = bevy_context.kayak_context.write() {
        context.render();

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
