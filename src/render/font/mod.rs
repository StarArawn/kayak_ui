use bevy::prelude::{Added, Entity, Plugin, Query, ResMut, Update};

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
            .add_systems(Update, process_loaded_fonts);
    }
}

fn process_loaded_fonts(
    mut font_mapping: ResMut<FontMapping>,
    context_query: Query<Entity, Added<KayakRootContext>>,
) {
    for _ in context_query.iter() {
        font_mapping.mark_all_as_new();
    }
}
