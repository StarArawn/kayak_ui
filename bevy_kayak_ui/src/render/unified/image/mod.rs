use bevy::prelude::Plugin;

mod extract;
mod image_manager;
pub use extract::extract_images;
pub use image_manager::ImageManager;

pub struct ImageRendererPlugin;

impl Plugin for ImageRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ImageManager::new());
    }
}
