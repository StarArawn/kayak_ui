//! Plugins and font renderers for the [Bevy] engine.
//!
//! [Bevy]: https://github.com/bevyengine/bevy

pub use loader::KayakFontLoader;
pub use plugin::KayakFontPlugin;
pub use renderer::*;

mod font_texture;
mod loader;
mod renderer;

mod plugin {
    use bevy::asset::AssetApp;
    use bevy::prelude::{IntoSystemConfigs, Plugin, Update};
    use bevy::render::{ExtractSchedule, Render, RenderApp, RenderSet};

    use crate::bevy::font_texture::init_font_texture;
    use crate::KayakFont;

    use super::*;

    pub struct KayakFontPlugin;

    impl Plugin for KayakFontPlugin {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.init_asset::<KayakFont>()
                .init_asset_loader::<crate::ttf::loader::TTFLoader>()
                .init_asset_loader::<KayakFontLoader>()
                .add_systems(Update, init_font_texture);

            let render_app = app.sub_app_mut(RenderApp);
            render_app
                .init_resource::<FontTextureCache>()
                .init_resource::<ExtractedFonts>()
                .add_systems(ExtractSchedule, extract_fonts)
                .add_systems(Render, prepare_fonts.in_set(RenderSet::Prepare));
        }
    }
}
