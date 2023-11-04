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
    use bevy::prelude::{AddAsset, IntoSystemConfigs, Plugin, Update};
    use bevy::render::{ExtractSchedule, Render, RenderApp, RenderSet};

    use crate::bevy::font_texture::init_font_texture;
    use crate::KayakFont;

    use super::*;

    pub struct KayakFontPlugin;

    impl Plugin for KayakFontPlugin {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.add_asset::<KayakFont>()
                .add_asset_loader(crate::ttf::loader::TTFLoader)
                .add_asset_loader(KayakFontLoader)
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
