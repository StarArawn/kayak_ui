use bevy::{
    prelude::{CoreStage, Plugin},
    render::camera::CameraTypePlugin,
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
        .add_plugin(CameraTypePlugin::<CameraUiKayak>::default());
    }
}
