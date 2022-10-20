use bevy::prelude::{Component, Entity, In, IntoSystem, System, World};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};

use crate::event::Event;
use crate::event_dispatcher::EventDispatcherContext;

/// A container for a function that handles events
///
/// This differs from a standard [`Handler`](crate::Handler) in that it's sent directly
/// from the [`KayakContext`](crate::KayakContext) and gives the [`KayakContextRef`]
/// as a parameter.
#[derive(Component, Clone)]
pub struct OnEvent {
    has_initialized: bool,
    system: Arc<
        RwLock<
            dyn System<
                In = (EventDispatcherContext, Event, Entity),
                Out = (EventDispatcherContext, Event),
            >,
        >,
    >,
}

impl Default for OnEvent {
    fn default() -> Self {
        Self::new(|In((event_dispatcher_context, event, _entity))| {
            (event_dispatcher_context, event)
        })
    }
}

impl OnEvent {
    /// Create a new event handler
    ///
    /// The handler should be a closure that takes the following arguments:
    /// 1. The current context
    /// 2. The event
    pub fn new<Params>(
        system: impl IntoSystem<
            (EventDispatcherContext, Event, Entity),
            (EventDispatcherContext, Event),
            Params,
        >,
    ) -> OnEvent {
        Self {
            has_initialized: false,
            system: Arc::new(RwLock::new(IntoSystem::into_system(system))),
        }
    }

    /// Call the event handler
    ///
    /// Returns true if the handler was successfully invoked.
    pub fn try_call(
        &mut self,
        mut event_dispatcher_context: EventDispatcherContext,
        entity: Entity,
        mut event: Event,
        world: &mut World,
    ) -> (EventDispatcherContext, Event) {
        if let Ok(mut system) = self.system.try_write() {
            if !self.has_initialized {
                system.initialize(world);
                self.has_initialized = true;
            }
            (event_dispatcher_context, event) =
                system.run((event_dispatcher_context, event, entity), world);
            system.apply_buffers(world);
        }
        (event_dispatcher_context, event)
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
