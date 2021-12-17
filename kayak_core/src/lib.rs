mod binding;
pub mod color;
pub mod context;
pub mod event;
pub mod fragment;
pub(crate) mod generational_arena;
mod input_event;
mod keys;
pub mod layout_cache;
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
pub use kayak_render_macros::{constructor, render, rsx, widget};
pub use keys::KeyCode;
pub use resources::Resources;
pub use tree::{Tree, WidgetTree};
pub use vec::VecTracker;
pub use widget::Widget;

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

// impl std::ops::Deref for OnEvent {
//     type Target =
//         Arc<RwLock<dyn FnMut(&mut crate::context::KayakContext, &mut Event) + Send + Sync>>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl std::ops::DerefMut for OnEvent {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

pub mod derivative {
    pub use derivative::*;
}
