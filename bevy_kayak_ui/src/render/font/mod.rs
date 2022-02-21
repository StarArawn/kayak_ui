use bevy::prelude::{Assets, Plugin, Res, ResMut};
use kayak_font::KayakFont;

mod extract;
mod font_mapping;

use crate::BevyContext;

pub use extract::extract_texts;
pub use font_mapping::*;

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
    fonts: Res<Assets<KayakFont>>,
    bevy_context: Option<Res<BevyContext>>,
) {
    if let Some(context) = bevy_context {
        if context.is_added() {
            font_mapping.mark_all_as_new();
        }
        font_mapping.add_loaded_to_kayak(&fonts, &context);
    }
}
