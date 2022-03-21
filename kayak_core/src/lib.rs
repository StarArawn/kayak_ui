mod assets;
mod binding;
mod children;
pub mod color;
pub mod context;
mod context_ref;
mod cursor;
mod cursor_icon;
pub mod event;
mod event_dispatcher;
mod flo_binding;
mod focus_tree;
pub mod fragment;
pub(crate) mod generational_arena;
mod input_event;
mod keyboard;
mod keys;
mod layout;
pub mod layout_cache;
mod layout_dispatcher;
mod multi_state;
pub mod node;
mod on_event;
mod on_layout;
pub mod render_command;
pub mod render_primitive;
pub mod styles;
pub mod tree;
mod vec;
pub mod widget;
pub mod widget_manager;

use std::sync::{Arc, RwLock};

pub use binding::*;
pub use children::Children;
pub use color::Color;
pub use context::*;
pub use context_ref::KayakContextRef;
pub use cursor::PointerEvents;
pub use cursor_icon::CursorIcon;
pub use event::*;
pub use focus_tree::FocusTree;
pub use fragment::{Fragment, FragmentProps};
pub use generational_arena::{Arena, Index};
pub use input_event::*;
pub use keyboard::{KeyboardEvent, KeyboardModifiers};
pub use keys::KeyCode;
pub use layout::*;
pub use on_event::OnEvent;
pub use on_layout::OnLayout;
pub use resources::Resources;
pub use tree::{Tree, WidgetTree};
pub use vec::{VecTracker, VecTrackerProps};
pub use widget::{BaseWidget, Widget, WidgetProps};

/// Type alias for dynamic widget objects. We use [BaseWidget] so that we can be object-safe
type BoxedWidget = Box<dyn BaseWidget>;

/// A simple handler object used for passing callbacks as props
///
/// # Examples
///
/// ```
/// # use kayak_core::Handler;
///
/// // Create a handler we can pass around
/// let on_select = Handler::new(|selected_index: usize| {
///   println!("Selected: {}", selected_index);
/// });
///
/// // Calling the handler can simply be done like
/// on_select.call(123);
/// ```
#[derive(Clone)]
pub struct Handler<T = ()>(pub Arc<RwLock<dyn FnMut(T) + Send + Sync + 'static>>);

impl<T> Default for Handler<T> {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(|_| {})))
    }
}

impl<T> Handler<T> {
    /// Create a new handler callback
    pub fn new<F: FnMut(T) + Send + Sync + 'static>(f: F) -> Handler<T> {
        Handler(Arc::new(RwLock::new(f)))
    }

    /// Call the handler
    ///
    /// # Panics
    ///
    /// Since the handler internally uses a `RwLock`, it can panic if the lock for the callback
    /// is already held by the current thread.
    pub fn call(&self, data: T) {
        if let Ok(mut handler) = self.0.write() {
            handler(data);
        }
    }
}

/// Always returns true for handlers of the same generic type
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
