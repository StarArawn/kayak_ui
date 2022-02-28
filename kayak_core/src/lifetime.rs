use crate::flo_binding::Uuid;
use crate::{Binding, Changeable, Releasable};
use std::collections::HashMap;

/// A container for storing callbacks tied to the lifetime of a widget
#[derive(Default)]
pub(crate) struct WidgetLifetime {
    /// Maps a [`Binding`] by ID to a callback function
    bindings: HashMap<Uuid, Box<dyn Releasable>>,
}

impl WidgetLifetime {
    /// Add a new callback for the given binding
    ///
    /// When the binding is changed via [`MutableBound`](crate::MutableBound), the given callback
    /// will be invoked. This callback will exist for the entire life of the widget or until it
    /// is removed via the [`remove`](Self::remove) or [`done`](Self::done) methods.
    ///
    /// # Arguments
    ///
    /// * `binding`: The binding to bind to
    /// * `callback`: The callback to invoke when the binding changes
    ///
    pub fn add<TBinding, TCallback>(&mut self, binding: &Binding<TBinding>, callback: TCallback)
    where
        TBinding: resources::Resource + Clone + PartialEq,
        TCallback: FnMut() -> () + Send + 'static,
    {
        let id = binding.id;
        if self.bindings.contains_key(&id) {
            // Binding already exists
            return;
        }
        let releasable = binding.when_changed(crate::notify(callback));
        self.bindings.insert(id, releasable);
    }

    /// Remove the callback for a given binding
    ///
    /// Returns the callback [`Releasable`] if it exists, otherwise `None`.
    ///
    /// # Arguments
    ///
    /// * `id`: The unique ID of the binding
    ///
    pub fn remove(&mut self, id: Uuid) -> Option<Box<dyn Releasable>> {
        self.bindings.remove(&id)
    }
}

impl std::fmt::Debug for WidgetLifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WidgetLifetime")
            .field("bindings", &self.bindings.keys())
            .finish()
    }
}
