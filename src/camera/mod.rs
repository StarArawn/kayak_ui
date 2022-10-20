use bevy::{
    prelude::{CoreStage, Plugin},
    render::{camera::CameraProjectionPlugin, extract_component::ExtractComponentPlugin},
};

mod camera;
mod ortho;

pub use camera::{CameraUiKayak, UICameraBundle};
pub(crate) use ortho::UIOrthographicProjection;

pub struct KayakUICameraPlugin;

impl Plugin for KayakUICameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            bevy::render::camera::camera_system::<UIOrthographicProjection>,
        )
        .add_plugin(CameraProjectionPlugin::<UIOrthographicProjection>::default())
        .add_plugin(ExtractComponentPlugin::<CameraUiKayak>::default());
    }
}
