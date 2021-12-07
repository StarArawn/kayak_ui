use bevy::{
    prelude::Plugin,
    render2::{RenderApp, RenderStage},
};

mod extract;
mod image_manager;
pub use image_manager::ImageManager;

pub struct ImageRendererPlugin;

impl Plugin for ImageRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ImageManager::new());

        let render_app = app.sub_app(RenderApp);
        render_app.add_system_to_stage(RenderStage::Extract, extract::extract_images);
    }
}
