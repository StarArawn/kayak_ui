use std::sync::{Arc, RwLock};

use kayak_core::context::KayakContext;

pub struct BevyContext {
    pub kayak_context: Arc<RwLock<KayakContext>>,
}

impl BevyContext {
    pub fn new<F: Fn(&mut KayakContext)>(f: F) -> Self {
        let kayak_context = Arc::new(RwLock::new(KayakContext::new()));

        if let Ok(mut kayak_context) = kayak_context.write() {
            f(&mut kayak_context);
            kayak_context.widget_manager.dirty(true);
        }

        Self { kayak_context }
    }
}
