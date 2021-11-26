use std::sync::{Arc, RwLock};

use kayak_core::{
    context::KayakContext,
    render_command::RenderCommand,
    styles::{Style, StyleProp, Units},
};

pub struct BevyContext {
    pub kayak_context: Arc<RwLock<KayakContext>>,
}

impl BevyContext {
    pub fn new<F: Fn(&mut Style, &mut KayakContext)>(width: f32, height: f32, f: F) -> Self {
        let mut app_styles = Style {
            render_command: StyleProp::Value(RenderCommand::Window),
            width: StyleProp::Value(Units::Pixels(width)),
            height: StyleProp::Value(Units::Pixels(height)),
            ..Style::default()
        };

        let kayak_context = Arc::new(RwLock::new(KayakContext::new()));

        if let Ok(mut kayak_context) = kayak_context.write() {
            f(&mut app_styles, &mut kayak_context);

            kayak_context.render();

            kayak_context.widget_manager.dirty(true);
        }

        Self { kayak_context }
    }
}
