use bevy::prelude::{Component, Entity, In, IntoSystem, System, World};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};

use crate::layout::LayoutEvent;

/// A container for a function that handles layout
///
/// This differs from a standard [`Handler`](crate::Handler) in that it's sent directly
/// from the [`KayakContext`](crate::KayakContext) and gives the [`KayakContextRef`]
/// as a parameter.
#[derive(Component, Clone)]
pub struct OnLayout {
    has_initialized: bool,
    system: Arc<RwLock<dyn System<In = (LayoutEvent, Entity), Out = LayoutEvent>>>,
}

impl Default for OnLayout {
    fn default() -> Self {
        Self::new(|In((event, _entity))| event)
    }
}

impl OnLayout {
    /// Create a new layout handler
    ///
    /// The handler should be a closure that takes the following arguments:
    /// 1. The LayoutEvent
    pub fn new<Params>(
        system: impl IntoSystem<(LayoutEvent, Entity), LayoutEvent, Params>,
    ) -> Self {
        Self {
            has_initialized: false,
            system: Arc::new(RwLock::new(IntoSystem::into_system(system))),
        }
    }

    /// Call the layout event handler
    ///
    /// Returns true if the handler was successfully invoked.
    pub fn try_call(
        &mut self,
        entity: Entity,
        mut event: LayoutEvent,
        world: &mut World,
    ) -> LayoutEvent {
        if let Ok(mut system) = self.system.try_write() {
            if !self.has_initialized {
                system.initialize(world);
                self.has_initialized = true;
            }
            event = system.run((event, entity), world);
            system.apply_deferred(world);
        }
        event
    }
}

impl Debug for OnLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnLayout").finish()
    }
}

impl PartialEq for OnLayout {
    fn eq(&self, _: &Self) -> bool {
        // Never prevent "==" for being true because of this struct
        true
    }
}
