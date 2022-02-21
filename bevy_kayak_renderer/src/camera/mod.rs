use bevy::{
    prelude::{CoreStage, Plugin},
    render::camera::ActiveCameras,
};

mod camera;
mod ortho;

pub use camera::UICameraBundle;
pub(crate) use ortho::UIOrthographicProjection;

pub struct KayakUICameraPlugin;

impl Plugin for KayakUICameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut active_cameras = app.world.get_resource_mut::<ActiveCameras>().unwrap();
        active_cameras.add(UICameraBundle::UI_CAMERA);
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            bevy::render::camera::camera_system::<UIOrthographicProjection>,
        );
    }
}
