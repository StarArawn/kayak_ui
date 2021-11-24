pub mod color;
pub mod context;
pub mod fragment;
pub mod layout_cache;
pub mod node;
pub mod render_command;
pub mod render_primitive;
pub mod styles;
pub mod tree;
pub mod widget;
pub mod widget_manager;

pub(crate) mod generational_arena;

pub use generational_arena::{Arena, Index};

pub use widget::Widget;

pub use kayak_render_macros::{render, rsx, widget};

pub use fragment::Fragment;

pub type Children = Option<
    std::sync::Arc<dyn Fn(Option<crate::Index>, &mut crate::context::KayakContext) + Send + Sync>,
>;

pub mod derivative {
    pub use derivative::*;
}
