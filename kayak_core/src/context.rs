use crate::{Changeable};
use std::collections::HashMap;
use flo_binding::{MutableBound, Releasable};

use crate::{multi_state::MultiState, widget_manager::WidgetManager, Index, InputEvent};
use crate::event_dispatcher::EventDispatcher;

pub struct KayakContext {
    widget_states: HashMap<crate::Index, resources::Resources>,
    widget_effects: HashMap<crate::Index, resources::Resources>,
    global_bindings: HashMap<crate::Index, Vec<flo_binding::Uuid>>,
    widget_state_lifetimes:
    HashMap<crate::Index, HashMap<flo_binding::Uuid, Box<dyn crate::Releasable>>>,
    current_id: Index,
    // TODO: Make widget_manager private.
    pub widget_manager: WidgetManager,
    event_dispatcher: EventDispatcher,
    global_state: resources::Resources,
    last_state_type_id: Option<std::any::TypeId>,
    current_state_index: usize,
    current_effect_index: usize,
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
            widget_states: HashMap::new(),
            widget_effects: HashMap::new(),
            global_bindings: HashMap::new(),
            widget_state_lifetimes: HashMap::new(),
            current_id: crate::Index::default(),
            widget_manager: WidgetManager::new(),
            event_dispatcher: EventDispatcher::default(),
            global_state: resources::Resources::default(),
            last_state_type_id: None,
            current_state_index: 0,
            current_effect_index: 0,
        }
    }

    /// Binds some global state to the current widget.
    pub fn bind<T: Clone + PartialEq + Send + Sync + 'static>(
        &mut self,
        global_state: &crate::Binding<T>,
    ) {
        if !self.global_bindings.contains_key(&self.current_id) {
            self.global_bindings.insert(self.current_id, vec![]);
        }

        let global_binding_ids = self.global_bindings.get_mut(&self.current_id).unwrap();

        if !global_binding_ids.contains(&global_state.id) {
            let cloned_id = self.current_id;
            let dirty_nodes = self.widget_manager.dirty_nodes.clone();
            let lifetime = global_state.when_changed(crate::notify(move || {
                if let Ok(mut dirty_nodes) = dirty_nodes.lock() {
                    dirty_nodes.insert(cloned_id);
                }
            }));
            Self::insert_state_lifetime(
                &mut self.widget_state_lifetimes,
                self.current_id,
                global_state.id,
                lifetime,
            );
            global_binding_ids.push(global_state.id);
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
                let dirty_nodes = self.widget_manager.dirty_nodes.clone();
                let cloned_id = self.current_id;
                let lifetime = state.when_changed(crate::notify(move || {
                    if let Ok(mut dirty_nodes) = dirty_nodes.lock() {
                        dirty_nodes.insert(cloned_id);
                    }
                }));
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
                let dirty_nodes = self.widget_manager.dirty_nodes.clone();
                let cloned_id = self.current_id;
                let lifetime = state.when_changed(crate::notify(move || {
                    if let Ok(mut dirty_nodes) = dirty_nodes.lock() {
                        dirty_nodes.insert(cloned_id);
                    }
                }));
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
            let dirty_nodes = self.widget_manager.dirty_nodes.clone();
            let cloned_id = self.current_id;
            let lifetime = state.when_changed(crate::notify(move || {
                if let Ok(mut dirty_nodes) = dirty_nodes.lock() {
                    dirty_nodes.insert(cloned_id);
                }
            }));
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
    pub fn create_effect<'a, F: Fn() + Send + Sync + 'static>(&'a mut self, effect: F, dependencies: &[&'a dyn Changeable]) {
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
        let effects = self.widget_effects.entry(self.current_id).or_insert(resources::Resources::default());
        if effects.contains::<MultiState<Vec<Box<dyn Releasable>>>>() {
            let mut state = effects.get_mut::<MultiState<Vec<Box<dyn Releasable>>>>().unwrap();
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

    fn insert_state_lifetime(
        lifetimes: &mut HashMap<
            crate::Index,
            HashMap<flo_binding::Uuid, Box<dyn crate::Releasable>>,
        >,
        id: Index,
        binding_id: flo_binding::Uuid,
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
            HashMap<flo_binding::Uuid, Box<dyn crate::Releasable>>,
        >,
        id: Index,
        binding_id: flo_binding::Uuid,
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
}
