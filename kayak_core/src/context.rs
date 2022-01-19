use crate::assets::AssetStorage;
use crate::{Binding, Changeable};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::event_dispatcher::EventDispatcher;
use crate::{
    multi_state::MultiState, widget_manager::WidgetManager, Index, InputEvent, MutableBound,
    Releasable,
};

pub struct KayakContext {
    assets: resources::Resources,
    current_effect_index: usize,
    current_id: Index,
    current_state_index: usize,
    event_dispatcher: EventDispatcher,
    global_bindings: HashMap<crate::Index, Vec<crate::flo_binding::Uuid>>,
    global_state: resources::Resources,
    last_state_type_id: Option<std::any::TypeId>,
    // TODO: Make widget_manager private.
    pub widget_manager: WidgetManager,
    widget_effects: HashMap<crate::Index, resources::Resources>,
    /// Contains provider state data to be accessed by consumers.
    ///
    /// Maps the type of the data to a mapping of the provider node's ID to the state data
    widget_providers: HashMap<std::any::TypeId, HashMap<crate::Index, resources::Resources>>,
    widget_state_lifetimes:
        HashMap<crate::Index, HashMap<crate::flo_binding::Uuid, Box<dyn crate::Releasable>>>,
    widget_states: HashMap<crate::Index, resources::Resources>,
}

impl std::fmt::Debug for KayakContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KayakContext")
            .field("current_id", &self.current_id)
            .finish()
    }
}

impl KayakContext {
    /// Creates a new [`KayakContext`].
    pub fn new() -> Self {
        Self {
            assets: resources::Resources::default(),
            current_effect_index: 0,
            current_id: crate::Index::default(),
            current_state_index: 0,
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

    /// Binds some global state to the current widget.
    pub fn bind<T: Clone + PartialEq + Send + Sync + 'static>(
        &mut self,
        binding: &crate::Binding<T>,
    ) {
        if !self.global_bindings.contains_key(&self.current_id) {
            self.global_bindings.insert(self.current_id, vec![]);
        }

        let global_binding_ids = self.global_bindings.get_mut(&self.current_id).unwrap();
        if !global_binding_ids.contains(&binding.id) {
            let lifetime = Self::create_lifetime(&binding, &self.widget_manager, self.current_id);
            Self::insert_state_lifetime(
                &mut self.widget_state_lifetimes,
                self.current_id,
                binding.id,
                lifetime,
            );
            global_binding_ids.push(binding.id);
        }
    }

    pub fn unbind<T: Clone + PartialEq + Send + Sync + 'static>(
        &mut self,
        global_state: &crate::Binding<T>,
    ) {
        if self.global_bindings.contains_key(&self.current_id) {
            let global_binding_ids = self.global_bindings.get_mut(&self.current_id).unwrap();
            if let Some(index) = global_binding_ids
                .iter()
                .position(|id| *id == global_state.id)
            {
                global_binding_ids.remove(index);

                Self::remove_state_lifetime(
                    &mut self.widget_state_lifetimes,
                    self.current_id,
                    global_state.id,
                );
            }
        }
    }

    /// Creates a provider context with the given state data
    ///
    /// This works much like [create_state](Self::create_state), except that the state is also made available to any children. They can
    /// access this provider's state by calling [create_consumer](Self::create_consumer).
    pub fn create_provider<T: resources::Resource + Clone + PartialEq>(
        &mut self,
        initial_state: T,
    ) -> Binding<T> {
        let type_id = initial_state.type_id();

        let providers = self
            .widget_providers
            .entry(type_id.clone())
            .or_insert(HashMap::default());

        if let Some(provider) = providers.get(&self.current_id) {
            if let Ok(state) = provider.get::<Binding<T>>() {
                // Provider was already created
                return state.clone();
            }
        }

        let mut provider = resources::Resources::default();
        let state = crate::bind(initial_state);
        let lifetime = Self::create_lifetime(&state, &self.widget_manager, self.current_id);
        Self::insert_state_lifetime(
            &mut self.widget_state_lifetimes,
            self.current_id,
            state.id,
            lifetime,
        );
        provider.insert(state.clone());
        providers.insert(self.current_id, provider);

        state
    }

    /// Creates a context consumer for the given type, [T]
    ///
    /// This allows direct access to a parent's state data made with [create_provider](Self::create_provider).
    pub fn create_consumer<T: resources::Resource + Clone + PartialEq>(
        &mut self,
    ) -> Option<Binding<T>> {
        let type_id = std::any::TypeId::of::<T>();

        if let Some(providers) = self.widget_providers.get(&type_id) {
            let mut index = Some(self.current_id);
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

    pub fn set_current_id(&mut self, id: crate::Index) {
        self.current_id = id;
        self.current_state_index = 0;
        self.current_effect_index = 0;
        self.last_state_type_id = None;
    }

    pub fn create_state<T: resources::Resource + Clone + PartialEq>(
        &mut self,
        initial_state: T,
    ) -> Option<crate::Binding<T>> {
        let state_type_id = initial_state.type_id();
        if let Some(last_state_type_id) = self.last_state_type_id {
            if state_type_id != last_state_type_id {
                self.current_state_index = 0;
            }
        }

        if self.widget_states.contains_key(&self.current_id) {
            let states = self.widget_states.get_mut(&self.current_id).unwrap();
            if !states.contains::<MultiState<crate::Binding<T>>>() {
                let state = crate::bind(initial_state);
                let lifetime = Self::create_lifetime(&state, &self.widget_manager, self.current_id);
                Self::insert_state_lifetime(
                    &mut self.widget_state_lifetimes,
                    self.current_id,
                    state.id,
                    lifetime,
                );
                states.insert(MultiState::new(state));
                self.last_state_type_id = Some(state_type_id);
                self.current_state_index += 1;
            } else {
                // Add new value to the multi-state.
                let state = crate::bind(initial_state);
                let lifetime = Self::create_lifetime(&state, &self.widget_manager, self.current_id);
                Self::insert_state_lifetime(
                    &mut self.widget_state_lifetimes,
                    self.current_id,
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
            let lifetime = Self::create_lifetime(&state, &self.widget_manager, self.current_id);
            Self::insert_state_lifetime(
                &mut self.widget_state_lifetimes,
                self.current_id,
                state.id,
                lifetime,
            );
            states.insert(MultiState::new(state));
            self.widget_states.insert(self.current_id, states);
            self.current_state_index += 1;
            self.last_state_type_id = Some(state_type_id);
        }
        return self.get_state();
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
    pub fn create_effect<'a, F: Fn() + Send + Sync + 'static>(
        &'a mut self,
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
            .entry(self.current_id)
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

    fn get_state<T: resources::Resource + Clone + PartialEq>(&self) -> Option<T> {
        if self.widget_states.contains_key(&self.current_id) {
            let states = self.widget_states.get(&self.current_id).unwrap();
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

    pub fn set_global_state<T: resources::Resource>(&mut self, state: T) {
        self.global_state.insert(state);
    }

    pub fn get_global_state<T: resources::Resource>(
        &mut self,
    ) -> Result<resources::RefMut<T>, resources::CantGetResource> {
        self.global_state.get_mut::<T>()
    }

    pub fn take_global_state<T: resources::Resource>(&mut self) -> Option<T> {
        self.global_state.remove::<T>()
    }

    pub fn render(&mut self) {
        let dirty_nodes: Vec<_> =
            if let Ok(mut dirty_nodes) = self.widget_manager.dirty_nodes.lock() {
                dirty_nodes.drain().collect()
            } else {
                panic!("Couldn't get lock on dirty nodes!")
            };
        for node_index in dirty_nodes {
            let mut widget = self.widget_manager.take(node_index);
            widget.render(self);
            self.widget_manager.repossess(widget);
            self.widget_manager.dirty_render_nodes.insert(node_index);
        }

        // self.widget_manager.dirty_nodes.clear();
        self.widget_manager.render();
        self.widget_manager.calculate_layout();
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
        self.event_dispatcher = dispatcher;
    }

    #[allow(dead_code)]
    fn get_all_parents(&self, current: Index, parents: &mut Vec<Index>) {
        if let Some(parent) = self.widget_manager.tree.parents.get(&current) {
            parents.push(*parent);
            self.get_all_parents(*parent, parents);
        }
    }

    pub fn is_focused(&self, index: Index) -> bool {
        let current = self.widget_manager.focus_tree.current();
        current == Some(index)
    }

    pub fn current_focus(&self) -> Option<Index> {
        self.widget_manager.focus_tree.current()
    }

    pub fn get_focusable(&self, index: Index) -> Option<bool> {
        self.widget_manager.get_focusable(index)
    }

    pub fn set_focusable(&mut self, focusable: Option<bool>, index: Index) {
        self.widget_manager.set_focusable(focusable, index, false);
    }

    /// Get the last calculated mouse position.
    ///
    /// Calling this from a widget will return the last mouse position at the time the widget was rendered.
    pub fn last_mouse_position(&self) -> (f32, f32) {
        self.event_dispatcher.current_mouse_position()
    }

    #[cfg(feature = "bevy_renderer")]
    pub fn query_world<T: bevy::ecs::system::SystemParam, F, R>(&mut self, mut f: F) -> R
        where
            F: FnMut(<T::Fetch as bevy::ecs::system::SystemParamFetch<'_, '_>>::Item) -> R,
    {
        let mut world = self.get_global_state::<bevy::prelude::World>().unwrap();
        let mut system_state = bevy::ecs::system::SystemState::<T>::new(&mut world);
        let r = {
            let test = system_state.get_mut(&mut world);
            f(test)
        };
        system_state.apply(&mut world);

        r
    }

    pub fn get_asset<T: 'static + Send + Sync + Clone + PartialEq>(
        &mut self,
        key: impl Into<PathBuf>,
    ) -> Binding<Option<T>> {
        self.create_asset_storage::<T>();
        if let Ok(mut asset_storage) = self.assets.get_mut::<AssetStorage<T>>() {
            asset_storage.get_asset(key).clone()
        } else {
            panic!("Couldn't find asset storage but it should exist!");
        }
    }

    pub fn set_asset<T: 'static + Send + Sync + Clone + PartialEq>(
        &mut self,
        key: impl Into<PathBuf>,
        asset: T,
    ) {
        self.create_asset_storage::<T>();
        if let Ok(mut asset_storage) = self.assets.get_mut::<AssetStorage<T>>() {
            asset_storage.set_asset(key, asset);
        } else {
            panic!("Couldn't find asset storage but it should exist!");
        }
    }

    fn create_asset_storage<T: 'static + Send + Sync + Clone + PartialEq>(&mut self) {
        if !self.assets.contains::<AssetStorage<T>>() {
            self.assets.insert(AssetStorage::<T>::new());
        }
    }

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
}
