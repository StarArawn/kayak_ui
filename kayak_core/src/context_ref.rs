use std::path::PathBuf;

use crate::{Binding, Changeable, Index, KayakContext, WidgetTree};

pub struct KayakContextRef<'a> {
    pub(crate) context: &'a mut KayakContext,
    current_id: Option<Index>,
    tree: Option<WidgetTree>,
}

impl<'a> KayakContextRef<'a> {
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

    pub fn bind<T: Clone + PartialEq + Send + Sync + 'static>(&mut self, binding: &Binding<T>) {
        self.context
            .bind(self.current_id.unwrap_or_default(), binding);
    }

    pub fn unbind<T: Clone + PartialEq + Send + Sync + 'static>(&mut self, binding: &Binding<T>) {
        self.context
            .unbind(self.current_id.unwrap_or_default(), binding);
    }

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

    pub fn create_state<T: resources::Resource + Clone + PartialEq>(
        &mut self,
        initial_state: T,
    ) -> Option<crate::Binding<T>> {
        self.context
            .create_state(self.current_id.unwrap_or_default(), initial_state)
    }

    /// Creates a callback that runs as a side-effect of its dependencies, running only when one of them is updated.
    ///
    /// All dependencies must be implement the [Changeable](crate::Changeable) trait, which means it will generally
    /// work best with [Binding](crate::Binding) values.
    ///
    /// For more details, check out [React's documentation](https://reactjs.org/docs/hooks-effect.html),
    /// upon which this method is based.
    ///
    /// # Arguments
    ///
    /// * `effect`: The side-effect function
    /// * `dependencies`: The dependencies the effect relies on
    ///
    /// returns: ()
    ///
    /// # Examples
    ///
    /// ```
    /// # use kayak_core::{bind, Binding, Bound, KayakContext};
    /// # let mut context = KayakContext::new();
    ///
    /// let my_state: Binding<i32> = bind(0i32);
    /// let my_state_clone = my_state.clone();
    /// context.create_effect(move || {
    ///     println!("Value: {}", my_state_clone.get());
    /// }, &[&my_state]);
    /// ```
    pub fn create_effect<'b, F: Fn() + Send + Sync + 'static>(
        &'b mut self,
        effect: F,
        dependencies: &[&'b dyn Changeable],
    ) {
        self.context
            .create_effect(self.current_id.unwrap_or_default(), effect, dependencies);
    }

    pub fn set_global_state<T: resources::Resource>(&mut self, state: T) {
        self.context.set_global_state(state);
    }

    pub fn get_global_state<T: resources::Resource>(
        &mut self,
    ) -> Result<resources::RefMut<T>, resources::CantGetResource> {
        self.context.get_global_state()
    }

    pub fn take_global_state<T: resources::Resource>(&mut self) -> Option<T> {
        self.context.take_global_state()
    }

    pub fn is_focused(&self, index: Index) -> bool {
        self.context.is_focused(index)
    }

    pub fn current_focus(&self) -> Option<Index> {
        self.context.current_focus()
    }

    pub fn get_focusable(&self, index: Index) -> Option<bool> {
        self.context.get_focusable(index)
    }

    pub fn set_focusable(&mut self, focusable: Option<bool>, index: Index) {
        self.context.set_focusable(focusable, index);
    }

    /// Get the last calculated mouse position.
    ///
    /// Calling this from a widget will return the last mouse position at the time the widget was rendered.
    pub fn last_mouse_position(&self) -> (f32, f32) {
        self.context.last_mouse_position()
    }

    #[cfg(feature = "bevy_renderer")]
    pub fn query_world<T: bevy::ecs::system::SystemParam, F, R>(&mut self, f: F) -> R
    where
        F: FnMut(<T::Fetch as bevy::ecs::system::SystemParamFetch<'_, '_>>::Item) -> R,
    {
        self.context.query_world::<T, F, R>(f)
    }

    pub fn get_asset<T: 'static + Send + Sync + Clone + PartialEq>(
        &mut self,
        key: impl Into<PathBuf>,
    ) -> Binding<Option<T>> {
        self.context.get_asset(key)
    }

    pub fn set_asset<T: 'static + Send + Sync + Clone + PartialEq>(
        &mut self,
        key: impl Into<PathBuf>,
        asset: T,
    ) {
        self.context.set_asset(key, asset)
    }

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

    pub fn get_valid_parent(&self, widget_id: Index) -> Option<Index> {
        self.context.widget_manager.get_valid_parent(widget_id)
    }

    pub fn get_layout(&self, widget_id: &Index) -> Option<&crate::layout_cache::Rect> {
        self.context.widget_manager.get_layout(widget_id)
    }

    pub fn get_node(&self, widget_id: &Index) -> Option<crate::node::Node> {
        self.context.widget_manager.get_node(widget_id)
    }

    pub fn get_name(&self, widget_id: &Index) -> Option<String> {
        self.context.widget_manager.get_name(widget_id)
    }

    /// Adds a widget to the context reference tree that will be committed to the main tree when `commit` is called.
    /// This also adds the widget to the `KayakContext` and renders the new widget.
    pub fn add_widget<W: crate::Widget + Clone + Default + PartialEq>(
        &mut self,
        widget: W,
        widget_index: usize,
    ) {
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
