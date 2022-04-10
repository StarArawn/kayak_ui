use std::path::PathBuf;

use crate::{Binding, Changeable, Index, KayakContext, WidgetTree};

/// A temporary struct used to provide limited access to the containing [`KayakContext`]
///
/// This provides a safer, cleaner way to access `KayakContext` when working with widgets.
pub struct KayakContextRef<'a> {
    /// A reference to the actual [`KayakContext`]
    pub(crate) context: &'a mut KayakContext,
    /// The ID of the current widget (usually the one this was passed to)
    current_id: Option<Index>,
    /// The currently generated widget tree
    tree: Option<WidgetTree>,
}

impl<'a> KayakContextRef<'a> {
    /// Creates a new `KayakContextRef`
    ///
    /// # Arguments
    ///
    /// * `context`: The containing `KayakContext`
    /// * `current_id`: The Id of the current widget
    ///
    pub fn new(context: &'a mut KayakContext, current_id: Option<Index>) -> Self {
        // TODO: Change this so that KayakContextRef keeps track of these instead of the KayakContext.
        context.last_state_type_id = None;
        context.current_state_index = 0;
        context.current_effect_index = 0;
        Self {
            context,
            current_id,
            tree: Some(WidgetTree::new()),
        }
    }

    /// Bind this widget to a `Binding<T>` value
    ///
    /// "Binding" means that whenever the bound value is changed, the current widget will be re-rendered.
    /// To undo this effect, use the [`unbind`](Self::unbind) method.
    ///
    /// Make sure the binding is stored _outside_ the widget's scope. Otherwise, it will just be dropped
    /// once the widget is rendered.
    ///
    /// # Arguments
    ///
    /// * `binding`: The value to bind to
    ///
    pub fn bind<T: Clone + PartialEq + Send + Sync + 'static>(&mut self, binding: &Binding<T>) {
        self.context
            .bind(self.current_id.unwrap_or_default(), binding);
    }

    /// Unbinds the current widget from a `Binding<T>` value
    ///
    /// The will only work on values for which the current widget has already been bound
    /// using the [`bind`](Self::bind) method.
    ///
    /// If the given value was not already bound, this method does nothing.
    ///
    /// # Arguments
    ///
    /// * `binding`: The already-bound value
    ///
    pub fn unbind<T: Clone + PartialEq + Send + Sync + 'static>(&mut self, binding: &Binding<T>) {
        self.context
            .unbind(self.current_id.unwrap_or_default(), binding);
    }

    /// Creates a provider context with the given state data
    ///
    /// This works much like [create_state](Self::create_state), except that the state is also made available to any
    /// descendent widget. They can access this provider's state by calling [create_consumer](Self::create_consumer).
    ///
    /// # Arguments
    ///
    /// * `initial_state`: The initial value to set (if it hasn't been set already)
    ///
    pub fn create_provider<T: resources::Resource + Clone + PartialEq>(
        &mut self,
        initial_state: T,
    ) -> Binding<T> {
        self.context
            .create_provider(self.current_id.unwrap_or_default(), initial_state)
    }

    /// Creates a context consumer for the given type, [T]
    ///
    /// This allows direct access to a parent's state data made with [create_provider](Self::create_provider).
    pub fn create_consumer<T: resources::Resource + Clone + PartialEq>(
        &mut self,
    ) -> Option<Binding<T>> {
        self.context
            .create_consumer(self.current_id.unwrap_or_default())
    }

    /// Create a state
    ///
    /// A "state" is a value that is maintained across re-renders of a widget. Additionally, widgets
    /// are _bound_ to their state. This means that whenever the state is updated, it will cause the
    /// widget to re-render.
    ///
    /// # Arguments
    ///
    /// * `initial_state`: The initial value to set (if it hasn't been set already)
    ///
    /// # Examples
    ///
    /// Creating a state is easy. With the `Bound` and `MutableBound` traits in scope, we can then
    /// `get` and `set` the state value, respectively.
    ///
    /// ```ignore
    /// #[widget]
    /// fn MyWidget() {
    ///   // Create state
    ///   let count = context.create_state::<u32>(0);
    ///
    ///   // Get current value
    ///   let count_value = count.get();
    ///
    ///   // Set value (this would cause the a re-render, resulting in an infinite loop)
    ///   count.set(count_value + 1);
    /// }
    /// ```
    ///
    /// The order in which states are defined matters. Placing this method behind some type of conditional
    /// can lead to unexpected behavior, such as one state being set to the value of another state.
    ///
    /// ```should_panic
    /// #[widget]
    /// fn MyWidget() {
    ///   let some_conditional = context.create_state(true);
    ///
    ///   if some_conditional {
    ///     let count_a = context.create_state::<u32>(123);
    ///     some_conditional.set(false);
    ///   }
    ///
    ///   let count_b = context.create_state::<u32>(0);
    ///
    ///   assert_eq!(0, count_b.get());
    /// }
    /// ```
    pub fn create_state<T: resources::Resource + Clone + PartialEq>(
        &mut self,
        initial_state: T,
    ) -> Option<crate::Binding<T>> {
        self.context
            .create_state(self.current_id.unwrap_or_default(), initial_state)
    }

    /// Creates a callback that runs as a side-effect of one of its dependencies being changed.
    ///
    /// All dependencies must be implement the [Changeable](crate::Changeable) trait, which means it will generally
    /// work best with [Binding](crate::Binding) values, such as those created by [`create_state`](Self::create_state).
    ///
    /// Use an empty dependency array if you want this effect to run only when the widget is _first_ rendered
    /// (then never again).
    ///
    /// For more details, check out [React's documentation](https://reactjs.org/docs/hooks-effect.html),
    /// upon which this method is based.
    ///
    /// # Arguments
    ///
    /// * `effect`: The side-effect function
    /// * `dependencies`: The dependencies the effect relies on
    ///
    /// # Examples
    ///
    /// ```ignore
    /// #[widget]
    /// fn MyWidget() {
    ///   let count = context.create_state::<u32>(0);
    ///
    ///   // An effect that prints out the count value whenever it changes
    ///   context.create_effect(move || {
    ///     println!("Value: {}", count.get());
    ///   }, &[&count]);
    ///
    ///   // An effect that prints to the console when the widget is first rendered
    ///   context.create_effect(|| {
    ///     println!("MyWidget created!");
    ///   }, &[]);
    /// }
    /// ```
    pub fn create_effect<'b, F: Fn() + Send + Sync + 'static>(
        &'b mut self,
        effect: F,
        dependencies: &[&'b dyn Changeable],
    ) {
        self.context
            .create_effect(self.current_id.unwrap_or_default(), effect, dependencies);
    }

    /// Set a value that's accessible to all widgets
    ///
    /// Values should be type-unique. Setting an `i32` value, for example, allows another widget
    /// to overwrite that value by adding their own global `i32` value, whether or not it was intentional.
    /// If this is not desired, an easy solution is to use the [newtype](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
    /// pattern.
    ///
    /// Widgets are not automatically bound to this global. You will have to bind to it manually
    /// (as long as the value is a `Binding<T>`) using [`bind`](Self::bind).
    ///
    /// # Arguments
    ///
    /// * `value`: The value to set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// struct MyCount(i32);
    ///
    /// #[widget]
    /// fn MyWidget() {
    ///   context.set_global(MyCount(123));
    /// }
    /// ```
    ///
    /// You may also want to bind the widget to a global, so that when the global is changed,
    /// the widget will re-render. This can be done by binding to the global.
    ///
    /// ```ignore
    /// use kayak_core::bind;
    ///
    /// #[derive(Clone, PartialEq)] // <- Required by `bind`
    /// struct MyCount(i32);
    ///
    /// #[widget]
    /// fn MyWidget() {
    ///   let bound_count = bind(MyCount(123));
    ///   context.bind(&bound_count);
    ///   context.set_global(bound_count);
    /// }
    /// ```
    pub fn set_global<T: resources::Resource>(&mut self, value: T) {
        self.context.set_global(value);
    }

    /// Attempts to fetch a global value with the given type, returning an immutable reference to
    /// that value.
    ///
    /// If you need mutable access to the global, use the [`get_global_mut`](Self::get_global_mut) method.
    pub fn get_global<T: resources::Resource>(
        &mut self,
    ) -> Result<resources::Ref<T>, resources::CantGetResource> {
        self.context.get_global()
    }

    /// Attempts to fetch a global value with the given type, returning a mutable reference to
    /// that value.
    ///
    /// If you only need immutable access to the global, use the [`get_global`](Self::get_global) method.
    pub fn get_global_mut<T: resources::Resource>(
        &mut self,
    ) -> Result<resources::RefMut<T>, resources::CantGetResource> {
        self.context.get_global_mut()
    }

    /// Removes the global value with the given type
    ///
    /// Returns the removed value, or `None` if a value with the given type did not exist.
    pub fn remove_global<T: resources::Resource>(&mut self) -> Option<T> {
        self.context.remove_global()
    }

    /// Checks if the widget with the given ID is currently focused or not
    pub fn is_focused(&self, id: Index) -> bool {
        self.context.is_focused(id)
    }

    /// Gets the currently focused widget ID
    pub fn current_focus(&self) -> Option<Index> {
        self.context.current_focus()
    }

    /// Gets whether the widget with the given ID can be focused
    ///
    /// The values are:
    ///
    /// | Value         | Description                              |
    /// |---------------|------------------------------------------|
    /// | `Some(true)`  | The widget is focusable                  |
    /// | `Some(false)` | The widget is not focusable              |
    /// | `None`        | The widget's focusability is unspecified |
    ///
    pub fn get_focusable(&self, id: Index) -> Option<bool> {
        self.context.get_focusable(id)
    }

    /// Sets the current widget's "focusability"
    ///
    /// The values are:
    ///
    /// | Value         | Description                              |
    /// |---------------|------------------------------------------|
    /// | `Some(true)`  | The widget is focusable                  |
    /// | `Some(false)` | The widget is not focusable              |
    /// | `None`        | The widget's focusability is unspecified |
    ///
    pub fn set_focusable(&mut self, focusable: Option<bool>) {
        if let Some(id) = self.current_id {
            self.context.set_focusable(focusable, id);
        }
    }

    /// Query the Bevy `World` with the given `SystemParam`
    ///
    /// The function passed to this method will be called with the retrieved value from `World`. If
    /// a value is returned from that function, it will be returned from this method as well.
    ///
    /// # Arguments
    ///
    /// * `f`: The function to call with the given system parameter
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use bevy::prelude::{Query, Res, Transform};
    ///
    /// struct MyCount(i32);
    ///
    /// #[widget]
    /// fn MyWidget() {
    ///   // Query a single item
    ///   let value = context.query_world::<Res<MyCount>, _, _>(|count| count.0);
    ///
    ///   // Or query multiple using a tuple
    ///   context.query_world::<(Res<MyCount>, Query<&mut Transform>), _, _>(|(count, query)| {
    ///     // ...
    ///   });
    /// }
    /// ```
    #[cfg(feature = "bevy_renderer")]
    pub fn query_world<T: bevy::ecs::system::SystemParam, F, R>(&mut self, f: F) -> R
    where
        F: FnMut(<T::Fetch as bevy::ecs::system::SystemParamFetch<'_, '_>>::Item) -> R,
    {
        self.context.query_world::<T, F, R>(f)
    }

    /// Get a stored asset with the given asset key
    ///
    /// The type of the asset [T] must implement `Clone` and `PartialEq` so that a `Binding<Option<T>>`
    /// can be returned. By calling [bind](Self::bind) over the binding, you can react to all changes to
    /// the asset, including when it's added or removed.
    ///
    /// If no asset in storage matches both the asset key _and_ the asset type, a value of
    /// `Binding<None>` is returned. Again, binding to this value will allow you to detect when a matching
    /// asset is added to storage.
    ///
    /// # Arguments
    ///
    /// * `key`: The asset key
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # #[derive(Clone, PartialEq)]
    /// # struct MyAsset(pub String);
    ///
    /// #[widget]
    /// fn MyWidget() {
    ///   let asset = context.get_asset::<MyAsset>("foo");
    ///   context.bind(&asset);
    ///   if let Some(asset) = asset.get() {
    ///     // ...
    ///   }
    /// }
    /// ```
    pub fn get_asset<T: 'static + Send + Sync + Clone + PartialEq>(
        &mut self,
        key: impl Into<PathBuf>,
    ) -> Binding<Option<T>> {
        self.context.get_asset(key)
    }

    /// Stores an asset along with a key to access it
    ///
    /// # Arguments
    ///
    /// * `key`: The asset key
    /// * `asset`: The asset to store
    ///
    pub fn set_asset<T: 'static + Send + Sync + Clone + PartialEq>(
        &mut self,
        key: impl Into<PathBuf>,
        asset: T,
    ) {
        self.context.set_asset(key, asset)
    }

    /// Get the last calculated mouse position.
    ///
    /// Calling this from a widget will return the last mouse position at the time the widget was rendered.
    pub fn last_mouse_position(&self) -> (f32, f32) {
        self.context.last_mouse_position()
    }

    /// Get the ID of the widget that was last clicked
    pub fn get_last_clicked_widget(&self) -> Binding<Index> {
        self.context.get_last_clicked_widget()
    }

    /// Returns true if the cursor is currently over a valid widget
    ///
    /// For the purposes of this method, a valid widget is one which has the means to display a visual component on its own.
    /// This means widgets specified with `RenderCommand::Empty`, `RenderCommand::Layout`, or `RenderCommand::Clip`
    /// do not meet the requirements to "contain" the cursor.
    pub fn contains_cursor(&self) -> bool {
        self.context.contains_cursor()
    }

    /// Returns true if the cursor may be needed by a widget or it's already in use by one
    ///
    /// This is useful for checking if certain events (such as a click) would "matter" to the UI at all. Example widgets
    /// include buttons, sliders, and text boxes.
    pub fn wants_cursor(&self) -> bool {
        self.context.wants_cursor()
    }

    /// Returns true if the cursor is currently in use by a widget
    ///
    /// This is most often useful for checking drag events as it will still return true even if the drag continues outside
    /// the widget bounds (as long as it started within it).
    pub fn has_cursor(&self) -> bool {
        self.context.has_cursor()
    }

    /// Captures all cursor events and instead makes the given index the target
    pub fn capture_cursor(&mut self, index: Index) -> Option<Index> {
        self.context.capture_cursor(index)
    }

    /// Releases the captured cursor
    ///
    /// Returns true if successful.
    ///
    /// This will only release the cursor if the given index matches the current captor. This
    /// prevents other widgets from accidentally releasing against the will of the original captor.
    ///
    /// This check can be side-stepped if necessary by calling [`force_release_cursor`](Self::force_release_cursor)
    /// instead (or by calling this method with the correct index).
    pub fn release_cursor(&mut self, index: Index) -> bool {
        self.context.release_cursor(index)
    }

    /// Releases the captured cursor
    ///
    /// Returns the index of the previous captor.
    ///
    /// This will force the release, regardless of which widget has called it. To safely release,
    /// use the standard [`release_cursor`](Self::release_cursor) method instead.
    pub fn force_release_cursor(&mut self) -> Option<Index> {
        self.context.force_release_cursor()
    }

    /// Attempts to get the parent of the widget with the given ID
    ///
    /// A "valid" parent is simply one that does not have a render command of
    /// [`RenderCommand::Empty`](crate::render_command::RenderCommand::Empty).
    ///
    /// # Arguments
    ///
    /// * `id`: The ID of the widget
    ///
    pub fn get_valid_parent(&self, id: Index) -> Option<Index> {
        self.context.widget_manager.get_valid_parent(id)
    }

    /// Attempts to get the children of the widget with the given ID
    ///
    /// A "valid" child is simply one that does not have a render command of
    /// [`RenderCommand::Empty`](crate::render_command::RenderCommand::Empty).
    ///
    /// # Arguments
    ///
    /// * `id`: The ID of the widget
    ///
    pub fn get_valid_children(&self, id: Index) -> Vec<Index> {
        self.context.widget_manager.get_valid_node_children(id)
    }

    /// Attempts to get the layout rect for the widget with the given ID
    ///
    /// # Arguments
    ///
    /// * `id`: The ID of the widget
    ///
    pub fn get_layout(&self, widget_id: &Index) -> Option<&crate::layout_cache::Rect> {
        self.context.widget_manager.get_layout(widget_id)
    }

    /// Get the render node for the widget with the given ID
    ///
    /// This is useful if you need access to the resolved styles, z-index, etc. of a widget.
    ///
    /// # Arguments
    ///
    /// * `id`: The ID of the widget
    ///
    pub fn get_node(&self, id: &Index) -> Option<crate::node::Node> {
        self.context.widget_manager.get_node(id)
    }

    /// Attempts to get the name of the widget with the given ID
    ///
    /// # Arguments
    ///
    /// * `id`: The ID of the widget
    ///
    pub fn get_name(&self, id: &Index) -> Option<String> {
        self.context.widget_manager.get_name(id)
    }

    /// Adds a widget to the context reference tree that will be committed to the main tree when `commit` is called.
    /// This also adds the widget to the `KayakContext` and renders the new widget.
    ///
    /// # Arguments
    ///
    /// * `widget`: The widget to add
    /// * `widget_index`: The widget's zero-based index amongst its siblings
    ///
    pub fn add_widget<W: crate::Widget>(&mut self, widget: W, widget_index: usize) {
        let (_, child_id) =
            self.context
                .widget_manager
                .create_widget(widget_index, widget, self.current_id);
        self.tree.as_ref().unwrap().add(child_id, self.current_id);

        let mut child_widget = self.context.widget_manager.take(child_id);
        {
            let mut context = KayakContextRef::new(&mut self.context, Some(child_id));
            // TODO: Use context ref here instead
            child_widget.render(&mut context);
        }
        self.context.widget_manager.repossess(child_widget);
    }

    /// Consumes the `KayakContextRef`. Internally this commits the newly built tree to the main widget tree.
    pub fn commit(&mut self) {
        // Consume the widget tree taking the inner value
        let tree = self.tree.take().unwrap().take();

        // Evaluate changes to the tree.
        let changes = self
            .context
            .widget_manager
            .tree
            .diff_children(&tree, self.current_id.unwrap_or_default());
        self.context
            .widget_manager
            .tree
            .merge(&tree, self.current_id.unwrap_or_default(), changes);
    }

    /// Marks the current widget as dirty (needing to be re-rendered)
    ///
    /// You should generally not need this. The point is to only re-render widgets when their state
    /// changes. This method bypasses that and forcibly re-renders the widget which can be wasteful if
    /// used improperly.
    ///
    /// Currently, this method is used internally for text rendering, which needs to re-render when
    /// it's parent layout is calculated.
    pub fn mark_dirty(&mut self) {
        if let Ok(mut dirty_nodes) = self.context.widget_manager.dirty_nodes.lock() {
            dirty_nodes.insert(self.current_id.unwrap_or_default());
        }
    }
}

#[test]
fn test_context_ref() {
    use crate::binding::{Bound, MutableBound};
    use crate::context_ref::KayakContextRef;

    let mut kayak_context = KayakContext::new();
    let mut kayak_context_ref = KayakContextRef::new(&mut kayak_context, None);

    let state = kayak_context_ref.create_state(0f32).unwrap();

    state.set(1.0);

    let state_value = state.get();
    assert!(state_value == 1.0);
}
