mod key;
mod pipeline;
mod plugin;

use std::sync::Arc;

use bevy::{
    asset::Asset,
    prelude::{Commands, Entity},
    reflect::Reflect,
    render::render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderRef},
};

pub use key::*;
pub use pipeline::*;
pub use plugin::*;

pub trait MaterialUI: AsBindGroup + Send + Sync + Clone + Asset + Sized {
    /// Returns this material's vertex shader. If [`ShaderRef::Default`] is returned, the default mesh vertex shader
    /// will be used.
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Returns this material's fragment shader. If [`ShaderRef::Default`] is returned, the default mesh fragment shader
    /// will be used.
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Customizes the default [`RenderPipelineDescriptor`].
    #[allow(unused_variables)]
    #[inline]
    fn specialize(descriptor: &mut RenderPipelineDescriptor, key: MaterialUIKey<Self>) {}
}

#[derive(Default, Clone, Reflect)]
pub struct MaterialHandle {
    uuid: String,
    #[reflect(ignore)]
    closure: HandleClosure,
}

#[derive(Clone)]
pub struct HandleClosure {
    pub(crate) c: Arc<dyn Fn(&mut Commands, Entity)>,
}

unsafe impl Send for HandleClosure {}
unsafe impl Sync for HandleClosure {}

impl Default for HandleClosure {
    fn default() -> Self {
        Self {
            c: Arc::new(|_, _| {}),
        }
    }
}

impl core::fmt::Debug for MaterialHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaterialHandle")
            .field("uuid", &self.uuid)
            .finish()
    }
}

impl PartialEq for MaterialHandle {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl MaterialHandle {
    pub fn new<F>(closure: F) -> Self
    where
        F: Fn(&mut Commands, Entity) + 'static,
    {
        Self {
            uuid: uuid::Uuid::new_v4().to_string(),
            closure: HandleClosure {
                c: Arc::new(closure),
            },
        }
    }

    pub fn run(&self, commands: &mut Commands, id: Entity) {
        self.closure.c.as_ref()(commands, id);
    }
}
