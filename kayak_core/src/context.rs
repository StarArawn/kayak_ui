use crate::assets::Assets;
use crate::layout_dispatcher::LayoutEventDispatcher;
use crate::{Binding, Changeable, CursorIcon, KayakContextRef};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::event_dispatcher::EventDispatcher;
use crate::{
    multi_state::MultiState, widget_manager::WidgetManager, Index, InputEvent, MutableBound,
    Releasable,
};

/// The context in which all widgets are contained
///
/// This manages everything from rendering widgets to processing events.
///
/// Generally widgets themselves do not need to interface with this struct directly
/// and can instead use the  [`KayakContextRef`] abstraction layer. Integrations, on
/// the other hand, will likely need to work with this struct directly so they can
/// control when to render, dispatch events, load assets, etc.
pub struct KayakContext {
    assets: Assets,
    pub(crate) current_effect_index: usize,
    pub(crate) current_state_index: usize,
    /// Processes and dispatches all events
    event_dispatcher: EventDispatcher,
    global_bindings: HashMap<crate::Index, Vec<crate::flo_binding::Uuid>>,
    global_state: resources::Resources,
    pub(crate) last_state_type_id: Option<std::any::TypeId>,
    // TODO: Make widget_manager private.
    /// The widget manager containing information about the widget tree and layout
    ///
    /// # Important Note
    ///
    /// While this is currently publicly accessible, it's recommended you __don't__ use it
    /// within your own code. This will likely be privatized in the future and only
    /// accessible through controlled layers of abstraction.
    pub widget_manager: WidgetManager,
    widget_effects: HashMap<crate::Index, resources::Resources>,
    /// Contains provider state data to be accessed by consumers.
    ///
    /// Maps the type of the data to a mapping of the provider node's ID to the state data
    widget_providers: HashMap<std::any::TypeId, HashMap<crate::Index, resources::Resources>>,
    widget_state_lifetimes:
        HashMap<crate::Index, HashMap<crate::flo_binding::Uuid, Box<dyn crate::Releasable>>>,
    widget_states: HashMap<crate::Index, resources::Resources>,
    cursor_icon: CursorIcon,
}

impl std::fmt::Debug for KayakContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KayakContext").finish()
    }
}

impl KayakContext {
    /// Creates a new [`KayakContext`].
    pub fn new() -> Self {
        Self {
            assets: Assets::default(),
            current_effect_index: 0,
            current_state_index: 0,
            cursor_icon: CursorIcon::Default,
            event_dispatcher: EventDispatcher::new(),
            global_bindings: HashMap::new(),
            global_state: resources::Resources::default(),
            last_state_type_id: None,
            widget_effects: HashMap::new(),
            widget_manager: WidgetManager::new(),
            widget_providers: HashMap::new(),
            widget_state_lifetimes: HashMap::new(),
            widget_states: HashMap::new(),
        }
    }

    /// Bind the given widget to a `Binding<T>` value
    ///
    /// "Binding" means that whenever the bound value is changed, the given widget will be re-rendered.
    /// To undo this effect, use the [`unbind`](Self::unbind) method.
    ///
    /// Make sure the binding is stored _outside_ the widget's scope. Otherwise, it will just be dropped
    /// once the widget is rendered.
    ///
    /// # Arguments
    ///
    /// * `widget_id`: The ID of the widget
    /// * `binding`: The value to bind to
    ///
    pub fn bind<T: Clone + PartialEq + Send + Sync + 'static>(
        &mut self,
        widget_id: Index,
        binding: &crate::Binding<T>,
    ) {
        if !self.global_bindings.contains_key(&widget_id) {
            self.global_bindings.insert(widget_id, vec![]);
        }

        let global_binding_ids = self.global_bindings.get_mut(&widget_id).unwrap();
        if !global_binding_ids.contains(&binding.id) {
            let lifetime = Self::create_lifetime(&binding, &self.widget_manager, widget_id);
            Self::insert_state_lifetime(
                &mut self.widget_state_lifetimes,
                widget_id,
                binding.id,
                lifetime,
            );
            global_binding_ids.push(binding.id);
        }
    }

    /// Unbinds the given widget from a `Binding<T>` value
    ///
    /// The will only work on values for which the given widget has already been bound
    /// using the [`bind`](Self::bind) method.
    ///
    /// If the given value was not already bound, this method does nothing.
    ///
    /// # Arguments
    ///
    /// * `widget_id`: The ID of the widget
    /// * `binding`: The already-bound value
    ///
    pub fn unbind<T: Clone + PartialEq + Send + Sync + 'static>(
        &mut self,
        widget_id: Index,
        binding: &crate::Binding<T>,
    ) {
        if self.global_bindings.contains_key(&widget_id) {
            let global_binding_ids = self.global_bindings.get_mut(&widget_id).unwrap();
            if let Some(index) = global_binding_ids.iter().position(|id| *id == binding.id) {
                global_binding_ids.remove(index);

                Self::remove_state_lifetime(
                    &mut self.widget_state_lifetimes,
                    widget_id,
                    binding.id,
                );
            }
        }
    }

    /// Creates a provider context with the given state data
    ///
    /// This works much like [create_state](Self::create_state), except that the state is also made available to any
    /// descendent widget. They can access this provider's state by calling [create_consumer](Self::create_consumer).
    ///
    /// # Arguments
    ///
    /// * `widget_id`: The ID of the widget
    /// * `initial_state`: The initial value to set (if it hasn't been set already)
    ///
    pub fn create_provider<T: resources::Resource + Clone + PartialEq>(
        &mut self,
        widget_id: Index,
        initial_state: T,
    ) -> Binding<T> {
        let type_id = initial_state.type_id();

        let providers = self
            .widget_providers
            .entry(type_id.clone())
            .or_insert(HashMap::default());

        if let Some(provider) = providers.get(&widget_id) {
            if let Ok(state) = provider.get::<Binding<T>>() {
                // Provider was already created
                return state.clone();
            }
        }

        let mut provider = resources::Resources::default();
        let state = crate::bind(initial_state);
        let lifetime = Self::create_lifetime(&state, &self.widget_manager, widget_id);
        Self::insert_state_lifetime(
            &mut self.widget_state_lifetimes,
            widget_id,
            state.id,
            lifetime,
        );
        provider.insert(state.clone());
        providers.insert(widget_id, provider);

        state
    }

    /// Creates a context consumer for the given type, [T]
    ///
    /// This allows direct access to a parent's state data made with [create_provider](Self::create_provider).
    ///
    /// # Arguments
    ///
    /// * `widget_id`: The ID of the widget
    ///
    pub fn create_consumer<T: resources::Resource + Clone + PartialEq>(
        &mut self,
        widget_id: Index,
    ) -> Option<Binding<T>> {
        let type_id = std::any::TypeId::of::<T>();

        if let Some(providers) = self.widget_providers.get(&type_id) {
            let mut index = Some(widget_id);
            while index.is_some() {
                // Traverse the parents to find the one with the given state data
                index = self.widget_manager.tree.get_parent(index.unwrap());

                if let Some(key) = index {
                    if let Some(provider) = providers.get(&key) {
                        if let Ok(state) = provider.get::<Binding<T>>() {
                            return Some(state.clone());
                        }
                    }
                }
            }
        }

        None
    }

    /// Create a state
    ///
    /// A "state" is a value that is maintained across re-renders of a widget. Additionally, widgets
    /// are _bound_ to their state. This means that whenever the state is updated, it will cause the
    /// widget to re-render.
    ///
    /// # Arguments
    ///
    /// * `widget_id`: The ID of the widget
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
        widget_id: Index,
        initial_state: T,
    ) -> Option<crate::Binding<T>> {
        let state_type_id = initial_state.type_id();
        if let Some(last_state_type_id) = self.last_state_type_id {
            if state_type_id != last_state_type_id {
                self.current_state_index = 0;
            }
        }

        if self.widget_states.contains_key(&widget_id) {
            let states = self.widget_states.get_mut(&widget_id).unwrap();
            if !states.contains::<MultiState<crate::Binding<T>>>() {
                let state = crate::bind(initial_state);
                let lifetime = Self::create_lifetime(&state, &self.widget_manager, widget_id);
                Self::insert_state_lifetime(
                    &mut self.widget_state_lifetimes,
                    widget_id,
                    state.id,
                    lifetime,
                );
                states.insert(MultiState::new(state));
                self.last_state_type_id = Some(state_type_id);
                self.current_state_index += 1;
            } else {
                // Add new value to the multi-state.
                let state = crate::bind(initial_state);
                let lifetime = Self::create_lifetime(&state, &self.widget_manager, widget_id);
                Self::insert_state_lifetime(
                    &mut self.widget_state_lifetimes,
                    widget_id,
                    state.id,
                    lifetime,
                );
                let mut multi_state = states.remove::<MultiState<crate::Binding<T>>>().unwrap();
                multi_state.get_or_add(state, &mut self.current_state_index);
                states.insert(multi_state);
                self.last_state_type_id = Some(state_type_id);
            }
        } else {
            let mut states = resources::Resources::default();
            let state = crate::bind(initial_state);
            let lifetime = Self::create_lifetime(&state, &self.widget_manager, widget_id);
            Self::insert_state_lifetime(
                &mut self.widget_state_lifetimes,
                widget_id,
                state.id,
                lifetime,
            );
            states.insert(MultiState::new(state));
            self.widget_states.insert(widget_id, states);
            self.current_state_index += 1;
            self.last_state_type_id = Some(state_type_id);
        }
        return self.get_state(widget_id);
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
    /// * `widget_id`: The ID of the widget
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
    pub fn create_effect<'a, F: Fn() + Send + Sync + 'static>(
        &'a mut self,
        widget_id: Index,
        effect: F,
        dependencies: &[&'a dyn Changeable],
    ) {
        // === Bind to Dependencies === //
        let notification = crate::notify(effect);
        let mut lifetimes = Vec::default();
        for dependency in dependencies {
            let lifetime = dependency.when_changed(notification.clone());
            lifetimes.push(lifetime);
        }

        // === Create Invoking Function === //
        // Create a temporary Binding to allow us to invoke the effect if needed
        let notify_clone = notification.clone();
        let invoke_effect = move || {
            let control = crate::bind(false);
            let mut control_life = control.when_changed(notify_clone.clone());
            control.set(true);
            control_life.done();
        };

        // === Insert Effect === //
        let effects = self
            .widget_effects
            .entry(widget_id)
            .or_insert(resources::Resources::default());
        if effects.contains::<MultiState<Vec<Box<dyn Releasable>>>>() {
            let mut state = effects
                .get_mut::<MultiState<Vec<Box<dyn Releasable>>>>()
                .unwrap();
            let old_size = state.data.len();
            state.get_or_add(lifetimes, &mut self.current_effect_index);
            if old_size != state.data.len() {
                // Just added -> invoke effect
                invoke_effect();
            }
        } else {
            let state = MultiState::new(lifetimes);
            effects.insert(state);
            invoke_effect();
            self.current_effect_index += 1;
        }
    }

    fn get_state<T: resources::Resource + Clone + PartialEq>(&self, widget_id: Index) -> Option<T> {
        if self.widget_states.contains_key(&widget_id) {
            let states = self.widget_states.get(&widget_id).unwrap();
            if let Ok(state) = states.get::<MultiState<T>>() {
                return Some(state.get(self.current_state_index - 1).clone());
            }
        }
        return None;
    }

    /// Create a `Releasable` lifetime that marks the current node as dirty when the given state changes
    fn create_lifetime<T: resources::Resource + Clone + PartialEq>(
        state: &Binding<T>,
        widget_manager: &WidgetManager,
        id: Index,
    ) -> Box<dyn Releasable> {
        let dirty_nodes = widget_manager.dirty_nodes.clone();
        state.when_changed(crate::notify(move || {
            if let Ok(mut dirty_nodes) = dirty_nodes.lock() {
                dirty_nodes.insert(id);
            }
        }))
    }

    fn insert_state_lifetime(
        lifetimes: &mut HashMap<
            crate::Index,
            HashMap<crate::flo_binding::Uuid, Box<dyn crate::Releasable>>,
        >,
        id: Index,
        binding_id: crate::flo_binding::Uuid,
        lifetime: Box<dyn crate::Releasable>,
    ) {
        if lifetimes.contains_key(&id) {
            if let Some(lifetimes) = lifetimes.get_mut(&id) {
                if !lifetimes.contains_key(&binding_id) {
                    lifetimes.insert(binding_id, lifetime);
                }
            }
        } else {
            let mut new_hashmap = HashMap::new();
            new_hashmap.insert(binding_id, lifetime);
            lifetimes.insert(id, new_hashmap);
        }
    }

    fn remove_state_lifetime(
        lifetimes: &mut HashMap<
            crate::Index,
            HashMap<crate::flo_binding::Uuid, Box<dyn crate::Releasable>>,
        >,
        id: Index,
        binding_id: crate::flo_binding::Uuid,
    ) {
        if lifetimes.contains_key(&id) {
            if let Some(lifetimes) = lifetimes.get_mut(&id) {
                if lifetimes.contains_key(&binding_id) {
                    let mut binding_lifetime = lifetimes.remove(&binding_id).unwrap();
                    binding_lifetime.done();
                }
            }
        }
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
        self.global_state.insert(value);
    }

    /// Attempts to fetch a global value with the given type, returning an immutable reference to
    /// that value.
    ///
    /// If you need mutable access to the global, use the [`get_global_mut`](Self::get_global_mut) method.
    pub fn get_global<T: resources::Resource>(
        &mut self,
    ) -> Result<resources::Ref<T>, resources::CantGetResource> {
        self.global_state.get::<T>()
    }

    /// Attempts to fetch a global value with the given type, returning a mutable reference to
    /// that value.
    ///
    /// If you only need immutable access to the global, use the [`get_global`](Self::get_global) method.
    pub fn get_global_mut<T: resources::Resource>(
        &mut self,
    ) -> Result<resources::RefMut<T>, resources::CantGetResource> {
        self.global_state.get_mut::<T>()
    }

    /// Removes the global value with the given type
    ///
    /// Returns the removed value, or `None` if a value with the given type did not exist.
    pub fn remove_global<T: resources::Resource>(&mut self) -> Option<T> {
        self.global_state.remove::<T>()
    }

    /// Re-render all widgets that need rendering (i.e., marked dirty)
    pub fn render(&mut self) {
        let dirty_nodes: Vec<_> =
            if let Ok(mut dirty_nodes) = self.widget_manager.dirty_nodes.lock() {
                dirty_nodes.drain(..).collect()
            } else {
                panic!("Couldn't get lock on dirty nodes!")
            };
        for node_index in dirty_nodes {
            let mut widget = self.widget_manager.take(node_index);
            let mut context = KayakContextRef::new(self, Some(node_index));
            widget.render(&mut context);
            self.widget_manager.repossess(widget);
            self.widget_manager.dirty_render_nodes.insert(node_index);
        }

        // self.widget_manager.dirty_nodes.clear();
        self.widget_manager.render(&mut self.assets);
        self.widget_manager.calculate_layout();
        LayoutEventDispatcher::dispatch(self);
        self.update_cursor();
    }

    /// Processes the given input events
    ///
    /// Events are processed in three phases: Capture, Target, Propagate. These phases are based on their
    /// associated [W3 specifications](https://www.w3.org/TR/uievents/#dom-event-architecture).
    ///
    /// ## Capture:
    /// Currently, we do not support the Capture Phase. This is because the current event handling system is
    ///   made to handle events as a single enum. To achieve proper capturing, widgets would need to be able to
    ///   register separate event handlers so that specific ones could be captured while others would not. It
    ///   should generally be okay to skip this as it's not a common use-case.
    ///
    /// ## Target:
    ///   The Target Phase simply identifies the target for an event so that we can generate the propagation path
    ///   for it.
    ///
    /// ## Propagate:
    ///   The Propagate Phase (also known as the Bubble Phase) is where we bubble up the tree from the target node,
    ///   firing the bubbled event along the way. At any point, the bubbling can be stopped by calling
    ///   [`event.stop_propagation()`](Event::stop_propagation). Not every event can be propagated, in which case,
    ///   they will only fire for their specified target.
    pub fn process_events(&mut self, input_events: Vec<InputEvent>) {
        let mut dispatcher = self.event_dispatcher.to_owned();
        dispatcher.process_events(input_events, self);
        self.event_dispatcher.merge(dispatcher);
    }

    #[allow(dead_code)]
    fn get_all_parents(&self, current: Index, parents: &mut Vec<Index>) {
        if let Some(parent) = self.widget_manager.tree.parents.get(&current) {
            parents.push(*parent);
            self.get_all_parents(*parent, parents);
        }
    }

    /// Checks if the widget with the given ID is currently focused or not
    pub fn is_focused(&self, index: Index) -> bool {
        let current = self.widget_manager.focus_tree.current();
        current == Some(index)
    }

    /// Gets the currently focused widget ID
    pub fn current_focus(&self) -> Option<Index> {
        self.widget_manager.focus_tree.current()
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
    pub fn get_focusable(&self, index: Index) -> Option<bool> {
        self.widget_manager.get_focusable(index)
    }

    /// Sets the "focusability" of the widget with the given ID
    ///
    /// The values are:
    ///
    /// | Value         | Description                              |
    /// |---------------|------------------------------------------|
    /// | `Some(true)`  | The widget is focusable                  |
    /// | `Some(false)` | The widget is not focusable              |
    /// | `None`        | The widget's focusability is unspecified |
    ///
    pub fn set_focusable(&mut self, focusable: Option<bool>, index: Index) {
        self.widget_manager.set_focusable(focusable, index, false);
    }

    /// Get the last calculated mouse position.
    ///
    /// Calling this from a widget will return the last mouse position at the time the widget was rendered.
    pub fn last_mouse_position(&self) -> (f32, f32) {
        self.event_dispatcher.current_mouse_position()
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
    pub fn query_world<T: bevy::ecs::system::SystemParam, F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(<T::Fetch as bevy::ecs::system::SystemParamFetch<'_, '_>>::Item) -> R,
    {
        let mut world = self.get_global_mut::<bevy::prelude::World>().unwrap();
        let mut system_state = bevy::ecs::system::SystemState::<T>::new(&mut world);
        let r = {
            let test = system_state.get_mut(&mut world);
            f(test)
        };
        system_state.apply(&mut world);

        r
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
    /// #[derive(Clone, PartialEq)]
    /// struct MyAsset(pub String);
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
        self.assets.get_asset(key)
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
        self.assets.set_asset(key, asset);
    }

    /// Get the ID of the widget that was last clicked
    pub fn get_last_clicked_widget(&self) -> Binding<Index> {
        self.event_dispatcher.last_clicked.clone()
    }

    /// Returns true if the cursor is currently over a valid widget
    ///
    /// For the purposes of this method, a valid widget is one which has the means to display a visual component on its own.
    /// This means widgets specified with `RenderCommand::Empty`, `RenderCommand::Layout`, or `RenderCommand::Clip`
    /// do not meet the requirements to "contain" the cursor.
    pub fn contains_cursor(&self) -> bool {
        self.event_dispatcher.contains_cursor()
    }

    /// Returns true if the cursor may be needed by a widget or it's already in use by one
    ///
    /// This is useful for checking if certain events (such as a click) would "matter" to the UI at all. Example widgets
    /// include buttons, sliders, and text boxes.
    pub fn wants_cursor(&self) -> bool {
        self.event_dispatcher.wants_cursor()
    }

    /// Returns true if the cursor is currently in use by a widget
    ///
    /// This is most often useful for checking drag events as it will still return true even if the drag continues outside
    /// the widget bounds (as long as it started within it).
    pub fn has_cursor(&self) -> bool {
        self.event_dispatcher.has_cursor()
    }

    /// Captures all cursor events and instead makes the given index the target
    pub fn capture_cursor(&mut self, index: Index) -> Option<Index> {
        self.event_dispatcher.capture_cursor(index)
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
        self.event_dispatcher.release_cursor(index)
    }

    /// Releases the captured cursor
    ///
    /// Returns the index of the previous captor.
    ///
    /// This will force the release, regardless of which widget has called it. To safely release,
    /// use the standard [`release_cursor`](Self::release_cursor) method instead.
    pub fn force_release_cursor(&mut self) -> Option<Index> {
        self.event_dispatcher.force_release_cursor()
    }

    /// Get the current cursor icon
    pub fn cursor_icon(&self) -> CursorIcon {
        self.cursor_icon
    }

    #[allow(dead_code)]
    pub(crate) fn set_cursor_icon(&mut self, icon: CursorIcon) {
        self.cursor_icon = icon;
    }

    fn update_cursor(&mut self) {
        if self.event_dispatcher.hovered.is_none() {
            return;
        }

        let hovered = self.event_dispatcher.hovered.unwrap();
        if let Some(node) = self.widget_manager.nodes.get(hovered) {
            if let Some(node) = node {
                let icon = node.resolved_styles.cursor.resolve();
                self.cursor_icon = icon;
            }
        }
    }
}
