use flo_binding::Changeable;
use std::collections::{HashMap, HashSet};

use crate::{widget_manager::WidgetManager, Event, EventType, Index, InputEvent};

pub struct KayakContext {
    widget_states: HashMap<crate::Index, resources::Resources>,
    global_bindings: HashMap<crate::Index, Vec<flo_binding::Uuid>>,
    widget_state_lifetimes:
    HashMap<crate::Index, HashMap<flo_binding::Uuid, Box<dyn crate::Releasable>>>,
    current_id: Index,
    pub widget_manager: WidgetManager,
    last_mouse_position: (f32, f32),
    is_mouse_pressed: bool,
    pub global_state: resources::Resources,
    previous_events: HashMap<Index, HashSet<EventType>>,
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
            global_bindings: HashMap::new(),
            widget_state_lifetimes: HashMap::new(),
            current_id: crate::Index::default(),
            widget_manager: WidgetManager::new(),
            last_mouse_position: (0.0, 0.0),
            is_mouse_pressed: false,
            global_state: resources::Resources::default(),
            previous_events: HashMap::new(),
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
    }

    pub fn create_state<T: resources::Resource + Clone + PartialEq>(
        &mut self,
        initial_state: T,
    ) -> Option<crate::Binding<T>> {
        if self.widget_states.contains_key(&self.current_id) {
            let states = self.widget_states.get_mut(&self.current_id).unwrap();
            if !states.contains::<crate::Binding<T>>() {
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
                states.insert(state);
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
            states.insert(state);
            self.widget_states.insert(self.current_id, states);
        }
        return self.get_state();
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

    fn get_state<T: resources::Resource + Clone + PartialEq>(&self) -> Option<T> {
        if self.widget_states.contains_key(&self.current_id) {
            let states = self.widget_states.get(&self.current_id).unwrap();
            if let Ok(state) = states.get::<T>() {
                return Some(state.clone());
            }
        }
        return None;
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

    pub fn process_events(&mut self, input_events: Vec<InputEvent>) {
        let mut events_stream = Vec::new();
        for (index, _) in self.widget_manager.nodes.iter() {
            if let Some(layout) = self.widget_manager.layout_cache.rect.get(&index) {
                for input_event in input_events.iter() {
                    match input_event {
                        InputEvent::MouseMoved(point) => {
                            // Hover event.
                            if layout.contains(point) {
                                if !Self::contains_event(&self.previous_events, &index, &EventType::MouseIn) {
                                    let mouse_in_event = Event {
                                        target: index,
                                        event_type: EventType::MouseIn,
                                        ..Event::default()
                                    };
                                    events_stream.push(mouse_in_event);
                                    Self::insert_event(
                                        &mut self.previous_events,
                                        &index,
                                        EventType::MouseIn,
                                    );
                                }
                                let hover_event = Event {
                                    target: index,
                                    event_type: EventType::Hover,
                                    ..Event::default()
                                };
                                events_stream.push(hover_event);

                                Self::insert_event(
                                    &mut self.previous_events,
                                    &index,
                                    EventType::Hover,
                                );
                            } else {
                                if Self::contains_event(&self.previous_events, &index, &EventType::Hover) ||
                                    Self::contains_event(&self.previous_events, &index, &EventType::MouseIn)
                                {
                                    let mouse_out_event = Event {
                                        target: index,
                                        event_type: EventType::MouseOut,
                                        ..Event::default()
                                    };
                                    events_stream.push(mouse_out_event);
                                    Self::insert_event(
                                        &mut self.previous_events,
                                        &index,
                                        EventType::MouseOut,
                                    );
                                    Self::remove_event(&mut self.previous_events, &index, &EventType::MouseIn);
                                    Self::remove_event(&mut self.previous_events, &index, &EventType::Hover);
                                } else {
                                    Self::remove_event(&mut self.previous_events, &index, &EventType::MouseOut);
                                }
                            }
                            self.last_mouse_position = *point;
                        }
                        InputEvent::MouseLeftPress => {
                            // Reset global mouse pressed
                            self.is_mouse_pressed = true;

                            if layout.contains(&self.last_mouse_position) {
                                let mouse_down_event = Event {
                                    target: index,
                                    event_type: EventType::MouseDown,
                                    ..Event::default()
                                };
                                events_stream.push(mouse_down_event);
                                Self::insert_event(
                                    &mut self.previous_events,
                                    &index,
                                    EventType::MouseDown,
                                );

                                // Start mouse pressed event
                                Self::insert_event(
                                    &mut self.previous_events,
                                    &index,
                                    EventType::MousePressed,
                                );
                            }
                        }
                        InputEvent::MouseLeftRelease => {
                            // Reset global mouse pressed
                            self.is_mouse_pressed = false;

                            if layout.contains(&self.last_mouse_position) {
                                let mouse_up_event = Event {
                                    target: index,
                                    event_type: EventType::MouseUp,
                                    ..Event::default()
                                };
                                events_stream.push(mouse_up_event);
                                Self::insert_event(
                                    &mut self.previous_events,
                                    &index,
                                    EventType::MouseUp,
                                );

                                if Self::contains_event(&self.previous_events, &index, &EventType::MousePressed) {
                                    let click_event = Event {
                                        target: index,
                                        event_type: EventType::Click,
                                        ..Event::default()
                                    };
                                    events_stream.push(click_event);
                                }
                            }
                        }
                    }
                }

                // Mouse is currently pressed for this node
                if self.is_mouse_pressed && Self::contains_event(&self.previous_events, &index, &EventType::MousePressed) {
                    let mouse_pressed_event = Event {
                        target: index,
                        event_type: EventType::MousePressed,
                        ..Event::default()
                    };
                    events_stream.push(mouse_pressed_event);
                } else {
                    Self::remove_event(&mut self.previous_events, &index, &EventType::MousePressed);
                }
            }
        }

        // Propagate Events
        for event in events_stream.iter_mut() {
            let mut parents: Vec<Index> = Vec::new();
            self.get_all_parents(event.target, &mut parents);

            // First call target
            let mut target_widget = self.widget_manager.take(event.target);
            target_widget.on_event(self, event);
            self.widget_manager.repossess(target_widget);

            for parent in parents {
                if event.should_propagate {
                    let mut parent_widget = self.widget_manager.take(parent);
                    parent_widget.on_event(self, event);
                    self.widget_manager.repossess(parent_widget);
                }
            }
        }
    }

    fn insert_event(
        previous_events: &mut HashMap<Index, HashSet<EventType>>,
        widget_id: &Index,
        event_type: EventType,
    ) -> bool {
        let mut entry = previous_events.entry(*widget_id).or_insert(HashSet::default());
        entry.insert(event_type)
    }

    fn remove_event(
        previous_events: &mut HashMap<Index, HashSet<EventType>>,
        widget_id: &Index,
        event_type: &EventType,
    ) -> bool {
        let mut entry = previous_events.entry(*widget_id).or_insert(HashSet::default());
        entry.remove(event_type)
    }

    fn reset_events(
        previous_events: &mut HashMap<Index, HashSet<EventType>>,
        widget_id: &Index,
    ) {
        let mut entry = previous_events.entry(*widget_id).or_insert(HashSet::default());
        entry.clear();
    }

    fn contains_event(
        previous_events: &HashMap<Index, HashSet<EventType>>,
        widget_id: &Index,
        event_type: &EventType,
    ) -> bool {
        if let Some(entry) = previous_events.get(widget_id) {
            entry.contains(event_type)
        } else {
            false
        }
    }

    fn has_any_event(
        previous_events: &HashMap<Index, HashSet<EventType>>,
        widget_id: &Index,
    ) -> bool {
        if let Some(entry) = previous_events.get(widget_id) {
            entry.len() > 0
        } else {
            false
        }
    }


    fn get_all_parents(&self, current: Index, parents: &mut Vec<Index>) {
        if let Some(parent) = self.widget_manager.tree.parents.get(&current) {
            parents.push(*parent);
            self.get_all_parents(*parent, parents);
        }
    }
}
