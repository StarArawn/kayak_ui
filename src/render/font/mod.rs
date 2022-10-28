use bevy::prelude::{Assets, Plugin, Res, ResMut};
use kayak_font::KayakFont;

mod extract;
mod font_mapping;

pub use extract::extract_texts;
pub use font_mapping::*;

use crate::context::KayakRootContext;

#[derive(Default)]
pub struct TextRendererPlugin;

impl Plugin for TextRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<FontMapping>()
            .add_system(process_loaded_fonts);
    }
}

fn process_loaded_fonts(
    mut font_mapping: ResMut<FontMapping>,
    _fonts: Res<Assets<KayakFont>>,
    context_resource: Res<KayakRootContext>,
) {
    // if let Some(context = context_resource.as_ref() {
    if context_resource.is_added() {
        font_mapping.mark_all_as_new();
    }
    // font_mapping.add_loaded_to_kayak(&fonts, &context);
    // }
}
