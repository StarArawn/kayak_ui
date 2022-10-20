use bevy::ecs::component::TableStorage;
use bevy::prelude::{Component, Entity, In, IntoSystem, System, World};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};

use crate::prelude::WidgetContext;

pub trait ChangeValue: Component<Storage = TableStorage> + Default {}

/// A container for a function that handles layout
///
/// This differs from a standard [`Handler`](crate::Handler) in that it's sent directly
/// from the [`KayakContext`](crate::KayakContext) and gives the [`KayakContextRef`]
/// as a parameter.
#[derive(Component, Clone)]
pub struct OnChange {
    value: Arc<RwLock<String>>,
    has_initialized: Arc<RwLock<bool>>,
    system: Arc<RwLock<dyn System<In = (WidgetContext, Entity, String), Out = ()>>>,
}

impl Default for OnChange {
    fn default() -> Self {
        Self::new(|In(_)| {})
    }
}

impl OnChange {
    /// Create a new layout handler
    ///
    /// The handler should be a closure that takes the following arguments:
    /// 1. The LayoutEvent
    pub fn new<Params>(
        system: impl IntoSystem<(WidgetContext, Entity, String), (), Params>,
    ) -> Self {
        Self {
            value: Default::default(),
            has_initialized: Arc::new(RwLock::new(false)),
            system: Arc::new(RwLock::new(IntoSystem::into_system(system))),
        }
    }

    pub fn set_value(&self, value: String) {
        if let Ok(mut value_mut) = self.value.try_write() {
            *value_mut = value;
        };
    }

    /// Call the layout event handler
    ///
    /// Returns true if the handler was successfully invoked.
    pub fn try_call(&self, entity: Entity, world: &mut World, widget_context: WidgetContext) {
        if let Ok(value) = self.value.try_read() {
            if let Ok(mut init) = self.has_initialized.try_write() {
                if let Ok(mut system) = self.system.try_write() {
                    if !*init {
                        system.initialize(world);
                        *init = true;
                    }
                    system.run((widget_context, entity, value.clone()), world);
                    system.apply_buffers(world);
                }
            }
        }
    }
}

impl Debug for OnChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnLayout").finish()
    }
}

impl PartialEq for OnChange {
    fn eq(&self, _: &Self) -> bool {
        // Never prevent "==" for being true because of this struct
        true
    }
}
