use crate::layout::LayoutEvent;
use crate::KayakContextRef;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};

/// A container for a function that handles layout
///
/// This differs from a standard [`Handler`](crate::Handler) in that it's sent directly
/// from the [`KayakContext`](crate::KayakContext) and gives the [`KayakContextRef`]
/// as a parameter.
#[derive(Clone)]
pub struct OnLayout(
    Arc<RwLock<dyn FnMut(&mut KayakContextRef, &LayoutEvent) + Send + Sync + 'static>>,
);

impl OnLayout {
    /// Create a new layout handler
    ///
    /// The handler should be a closure that takes the following arguments:
    /// 1. The current context
    /// 2. The LayoutEvent
    pub fn new<F: FnMut(&mut KayakContextRef, &LayoutEvent) + Send + Sync + 'static>(
        f: F,
    ) -> OnLayout {
        OnLayout(Arc::new(RwLock::new(f)))
    }

    /// Call the layout handler
    ///
    /// Returns true if the handler was successfully invoked.
    pub fn try_call(&self, context: &mut KayakContextRef, event: &LayoutEvent) -> bool {
        if let Ok(mut on_layout) = self.0.write() {
            on_layout(context, event);
            true
        } else {
            false
        }
    }
}

impl Debug for OnLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnLayout").finish()
    }
}

impl PartialEq for OnLayout {
    fn eq(&self, _: &Self) -> bool {
        // Never prevent "==" from being true because of this struct
        true
    }
}
