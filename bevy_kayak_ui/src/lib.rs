use bevy::{prelude::{Plugin, ResMut}, render2::color::Color};

mod bevy_context;
mod camera;
mod render;

pub use bevy_context::BevyContext;
pub use camera::*;

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

pub fn update(bevy_context: ResMut<BevyContext>) {
    if let Ok(mut context) = bevy_context.kayak_context.write() {
        context.render();
    }
}
