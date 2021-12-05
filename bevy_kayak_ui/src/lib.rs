use bevy::{
    input::{mouse::MouseButtonInput, ElementState},
    math::Vec2,
    prelude::{
        EventReader, IntoExclusiveSystem, IntoSystem, MouseButton, Mut, Plugin, Res, ResMut, World,
    },
    render2::color::Color,
    window::{CursorMoved, Windows},
};

mod bevy_context;
mod camera;
mod render;

pub use bevy_context::BevyContext;
pub use camera::*;
use kayak_core::{context::GlobalState, InputEvent};

#[derive(Default)]
pub struct BevyKayakUIPlugin;

impl Plugin for BevyKayakUIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(render::BevyKayakUIRenderPlugin)
            .add_plugin(camera::KayakUICameraPlugin)
            .add_system(update.exclusive_system());
    }
}

pub(crate) fn to_bevy_color(color: &kayak_core::color::Color) -> Color {
    Color::rgba(color.r, color.g, color.b, color.a)
}

pub struct WrappedWorld<'a> {
    world: &'a mut World,
}

impl<'a> GlobalState for WrappedWorld<'a> {}

pub fn update(world: &mut World) {
    let window_size = {
        let windows = world.get_resource::<Windows>().unwrap();
        if let Some(window) = windows.get_primary() {
            Vec2::new(window.width(), window.height())
        } else {
            panic!("Couldn't find primary window!");
        }
    };

    let bevy_context = world.remove_resource::<BevyContext<'static>>().unwrap();
    if let Ok(mut context) = bevy_context.kayak_context.write() {
        let mut wrapped_world = WrappedWorld { world };
        context.render(&mut wrapped_world);
    }

    if let Ok(mut context) = bevy_context.kayak_context.write() {
        let mut input_events = Vec::new();

        {
            let mut cursor_moved_events = world
                .get_resource_mut::<EventReader<CursorMoved>>()
                .unwrap();
            for event in cursor_moved_events.iter() {
                input_events.push(InputEvent::MouseMoved((
                    event.position.x as f32,
                    window_size.y - event.position.y as f32,
                )));
            }
        }

        {
            let mut mouse_button_input_events = world
                .get_resource_mut::<EventReader<MouseButtonInput>>()
                .unwrap();
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
        }

        context.process_events(input_events);
    }

    world.insert_resource(bevy_context);
}
