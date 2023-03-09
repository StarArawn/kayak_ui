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
    use bevy::prelude::{AddAsset, IntoSystemAppConfig, IntoSystemConfig, Plugin};
    use bevy::render::{ExtractSchedule, RenderApp, RenderSet};

    use crate::bevy::font_texture::init_font_texture;
    use crate::KayakFont;

    use super::*;

    pub struct KayakFontPlugin;

    impl Plugin for KayakFontPlugin {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.add_asset::<KayakFont>()
                .add_asset_loader(crate::ttf::loader::TTFLoader)
                .add_asset_loader(KayakFontLoader)
                .add_system(init_font_texture);

            let render_app = app.sub_app_mut(RenderApp);
            render_app
                .init_resource::<FontTextureCache>()
                .init_resource::<ExtractedFonts>()
                .add_system(extract_fonts.in_schedule(ExtractSchedule))
                .add_system(prepare_fonts.in_set(RenderSet::Prepare));
        }
    }
}
