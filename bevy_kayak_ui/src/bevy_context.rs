use std::sync::{Arc, RwLock};

use kayak_core::context::KayakContext;

/// A wrapper around `KayakContext` to be used in Bevy integrations
///
/// ```
/// use bevy::prelude::*;
/// use bevy_kayak_ui::BevyContext;
///
/// fn ui_system(context: Res<BevyContext>) {
///   // ...
/// }
/// ```
pub struct BevyContext {
    pub kayak_context: Arc<RwLock<KayakContext>>,
}

impl BevyContext {
    /// Create a new `BevyContext`
    ///
    /// This takes a function that will setup the `KayakContext` and its widget tree.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_kayak_ui::BevyContext;
    ///
    /// fn setup_context(mut commands: Commands) {
    ///   let context = BevyContext::new(|context| {
    ///     render! {
    ///       <>
    ///         // ...
    ///       </>
    ///     }
    ///   });
    ///
    ///   commands.insert_resource(context);
    /// }
    /// ```
    pub fn new<F: Fn(&mut KayakContext)>(f: F) -> Self {
        let kayak_context = Arc::new(RwLock::new(KayakContext::new()));

        if let Ok(mut kayak_context) = kayak_context.write() {
            f(&mut kayak_context);
            kayak_context.widget_manager.dirty(true);
        }

        Self { kayak_context }
    }

    /// Returns true if the cursor is currently over a valid widget
    ///
    /// For the purposes of this method, a valid widget is one which has the means to display a visual component on its own.
    /// This means widgets specified with `RenderCommand::Empty`, `RenderCommand::Layout`, or `RenderCommand::Clip`
    /// do not meet the requirements to "contain" the cursor.
    pub fn contains_cursor(&self) -> bool {
        if let Ok(kayak_context) = self.kayak_context.read() {
            kayak_context.contains_cursor()
        } else {
            false
        }
    }

    /// Returns true if the cursor may be needed by a widget or it's already in use by one
    ///
    /// This is useful for checking if certain events (such as a click) would "matter" to the UI at all. Example widgets
    /// include buttons, sliders, and text boxes.
    pub fn wants_cursor(&self) -> bool {
        if let Ok(kayak_context) = self.kayak_context.read() {
            kayak_context.wants_cursor()
        } else {
            false
        }
    }

    /// Returns true if the cursor is currently in use by a widget
    ///
    /// This is most often useful for checking drag events as it will still return true even if the drag continues outside
    /// the widget bounds (as long as it started within it).
    pub fn has_cursor(&self) -> bool {
        if let Ok(kayak_context) = self.kayak_context.read() {
            kayak_context.has_cursor()
        } else {
            false
        }
    }
}
