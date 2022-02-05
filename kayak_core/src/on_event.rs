use crate::{Event, KayakContextRef};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};

/// A container for a function that handles events
#[derive(Clone)]
pub struct OnEvent(
    Arc<RwLock<dyn FnMut(&mut KayakContextRef, &mut Event) + Send + Sync + 'static>>,
);

impl OnEvent {
    /// Create a new event handler
    ///
    /// The handler should be a closure that takes the following arguments:
    /// 1. The current context
    /// 2. The event
    pub fn new<F: FnMut(&mut KayakContextRef, &mut Event) + Send + Sync + 'static>(
        f: F,
    ) -> OnEvent {
        OnEvent(Arc::new(RwLock::new(f)))
    }

    /// Call the event handler
    ///
    /// Returns true if the handler was successfully invoked.
    pub fn try_call(&self, context: &mut KayakContextRef, event: &mut Event) -> bool {
        if let Ok(mut on_event) = self.0.write() {
            on_event(context, event);
            true
        } else {
            false
        }
    }
}

impl Debug for OnEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnEvent").finish()
    }
}

impl PartialEq for OnEvent {
    fn eq(&self, _: &Self) -> bool {
        // Never prevent "==" for being true because of this struct
        true
    }
}
