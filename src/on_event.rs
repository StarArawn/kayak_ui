use bevy::prelude::{Component, Entity, In, IntoSystem, System, World};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};

use crate::event::KEvent;
use crate::event_dispatcher::EventDispatcherContext;
use crate::focus_tree::FocusTree;
use crate::widget_state::WidgetState;

/// A container for a function that handles events
///
/// This differs from a standard [`Handler`](crate::Handler) in that it's sent directly
/// from the [`KayakContext`](crate::KayakContext) and gives the [`KayakContextRef`]
/// as a parameter.
#[derive(Component, Clone)]
pub struct OnEvent {
    has_initialized: Arc<RwLock<bool>>,
    system: Arc<RwLock<dyn System<In = Entity, Out = ()>>>,
}

impl Default for OnEvent {
    fn default() -> Self {
        Self::new(|In(_entity)| {})
    }
}
impl OnEvent {
    /// Create a new event handler
    ///
    /// The handler should be a closure that takes the following arguments:
    /// 1. The current context
    /// 2. The event
    pub fn new<Params>(system: impl IntoSystem<Entity, (), Params>) -> OnEvent {
        Self {
            has_initialized: Arc::new(RwLock::new(false)),
            system: Arc::new(RwLock::new(IntoSystem::into_system(system))),
        }
    }

    /// Call the event handler
    ///
    /// Returns true if the handler was successfully invoked.
    pub fn try_call(
        &mut self,
        mut event_dispatcher_context: EventDispatcherContext,
        widget_state: WidgetState,
        focus_tree: FocusTree,
        entity: Entity,
        mut event: KEvent,
        world: &mut World,
    ) -> (EventDispatcherContext, KEvent) {
        if let Ok(mut system) = self.system.try_write() {
            if let Ok(mut has_initialized) = self.has_initialized.try_write() {
                if !*has_initialized {
                    system.initialize(world);
                    *has_initialized = true;
                }
            }
            // Insert resources
            world.insert_resource(event_dispatcher_context);
            world.insert_resource(widget_state);
            world.insert_resource(event);
            world.insert_resource(focus_tree);

            system.run(entity, world);
            system.apply_buffers(world);

            event_dispatcher_context = world.remove_resource::<EventDispatcherContext>().unwrap();
            event = world.remove_resource::<KEvent>().unwrap();
            world.remove_resource::<WidgetState>().unwrap();
            world.remove_resource::<FocusTree>().unwrap();
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
