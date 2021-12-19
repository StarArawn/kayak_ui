mod binding;
pub mod color;
pub mod context;
pub mod event;
pub mod fragment;
pub(crate) mod generational_arena;
mod input_event;
mod keys;
pub mod layout_cache;
mod multi_state;
pub mod node;
pub mod render_command;
pub mod render_primitive;
pub mod styles;
pub mod tree;
mod vec;
pub mod widget;
pub mod widget_manager;

use std::sync::{Arc, RwLock};

pub use binding::*;
pub use color::Color;
pub use context::*;
pub use event::*;
pub use fragment::Fragment;
pub use generational_arena::{Arena, Index};
pub use input_event::*;
pub use kayak_render_macros::{constructor, render, rsx, use_state, widget};
pub use keys::KeyCode;
pub use resources::Resources;
pub use tree::{Tree, WidgetTree};
pub use vec::VecTracker;
pub use widget::Widget;
pub mod derivative {
    pub use derivative::*;
}

pub type Children = Option<
    Arc<dyn Fn(WidgetTree, Option<crate::Index>, &mut crate::context::KayakContext) + Send + Sync>,
>;

#[derive(Clone)]
pub struct OnEvent(
    pub  Arc<
        RwLock<dyn FnMut(&mut crate::context::KayakContext, &mut Event) + Send + Sync + 'static>,
    >,
);

impl OnEvent {
    pub fn new<F: FnMut(&mut crate::context::KayakContext, &mut Event) + Send + Sync + 'static>(
        f: F,
    ) -> OnEvent {
        OnEvent(Arc::new(RwLock::new(f)))
    }
}

#[derive(Clone)]
pub struct Handler<T>(pub Arc<RwLock<dyn FnMut(T) + Send + Sync + 'static>>);

impl<T> Handler<T> {
    pub fn new<F: FnMut(T) + Send + Sync + 'static>(f: F) -> Handler<T> {
        Handler(Arc::new(RwLock::new(f)))
    }

    pub fn call(&self, data: T) {
        if let Ok(mut handler) = self.0.write() {
            handler(data);
        }
    }
}

impl<T> PartialEq for Handler<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> std::fmt::Debug for Handler<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Handler").finish()
    }
}
