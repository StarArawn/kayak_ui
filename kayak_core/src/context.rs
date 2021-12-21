use flo_binding::Changeable;
use morphorm::Hierarchy;
use std::collections::{HashMap, HashSet};

use crate::{
    multi_state::MultiState, widget_manager::WidgetManager, Event, EventType, Index, InputEvent,
};

pub struct KayakContext {
    widget_states: HashMap<crate::Index, resources::Resources>,
    global_bindings: HashMap<crate::Index, Vec<flo_binding::Uuid>>,
    widget_state_lifetimes:
        HashMap<crate::Index, HashMap<flo_binding::Uuid, Box<dyn crate::Releasable>>>,
    current_id: Index,
    // TODO: Make widget_manager private.
    pub widget_manager: WidgetManager,
    last_mouse_position: (f32, f32),
    is_mouse_pressed: bool,
    previous_events: HashMap<Index, HashSet<EventType>>,
    global_state: resources::Resources,
    current_focus: Index,
    last_focus: Index,
    last_state_type_id: Option<std::any::TypeId>,
    current_state_index: usize,
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
            current_focus: Index::default(),
            last_focus: Index::default(),
            last_state_type_id: None,
            current_state_index: 0,
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

    pub fn process_events(&mut self, input_events: Vec<InputEvent>) {
        let mut events_stream = Vec::new();
        let mut next_events = HashMap::default();

        let mut was_mouse_down_event = false;
        let mut was_focus_event = false;

        for index in self.widget_manager.node_tree.down_iter() {
            if let Some(layout) = self.widget_manager.layout_cache.rect.get(&index) {
                for input_event in input_events.iter() {
                    match input_event {
                        InputEvent::MouseMoved(point) => {
                            // Hover event.
                            if layout.contains(point) {
                                if !Self::contains_event(
                                    &self.previous_events,
                                    &index,
                                    &EventType::MouseIn,
                                ) {
                                    let mouse_in_event = Event {
                                        target: index,
                                        event_type: EventType::MouseIn,
                                        ..Event::default()
                                    };
                                    events_stream.push(mouse_in_event);
                                    Self::insert_event(
                                        &mut next_events,
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
                                Self::insert_event(&mut next_events, &index, EventType::Hover);
                            } else {
                                if Self::contains_event(
                                    &self.previous_events,
                                    &index,
                                    &EventType::Hover,
                                ) || Self::contains_event(
                                    &self.previous_events,
                                    &index,
                                    &EventType::MouseIn,
                                ) {
                                    let mouse_out_event = Event {
                                        target: index,
                                        event_type: EventType::MouseOut,
                                        ..Event::default()
                                    };
                                    events_stream.push(mouse_out_event);
                                    Self::insert_event(
                                        &mut next_events,
                                        &index,
                                        EventType::MouseOut,
                                    );
                                }
                            }
                            self.last_mouse_position = *point;
                        }
                        InputEvent::MouseLeftPress => {
                            // Reset global mouse pressed
                            self.is_mouse_pressed = true;
                            was_mouse_down_event = true;

                            if layout.contains(&self.last_mouse_position) {
                                let mouse_down_event = Event {
                                    target: index,
                                    event_type: EventType::MouseDown,
                                    ..Event::default()
                                };
                                events_stream.push(mouse_down_event);
                                Self::insert_event(&mut next_events, &index, EventType::MouseDown);

                                // Start mouse pressed event as well
                                Self::insert_event(
                                    &mut next_events,
                                    &index,
                                    EventType::MousePressed,
                                );

                                if let Some(widget) =
                                    self.widget_manager.current_widgets.get(index).unwrap()
                                {
                                    if widget.focusable() {
                                        was_focus_event = true;
                                        let focus_event = Event {
                                            target: index,
                                            event_type: EventType::Focus,
                                            ..Event::default()
                                        };
                                        events_stream.push(focus_event);
                                        self.last_focus = self.current_focus;
                                        self.current_focus = index;
                                    }
                                }
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
                                Self::insert_event(&mut next_events, &index, EventType::MouseUp);

                                if Self::contains_event(
                                    &self.previous_events,
                                    &index,
                                    &EventType::MousePressed,
                                ) {
                                    let click_event = Event {
                                        target: index,
                                        event_type: EventType::Click,
                                        ..Event::default()
                                    };
                                    events_stream.push(click_event);
                                    Self::insert_event(&mut next_events, &index, EventType::Click);
                                }
                            }
                        }
                        InputEvent::CharEvent { c } => events_stream.push(Event {
                            target: index,
                            event_type: EventType::CharInput { c: *c },
                            ..Event::default()
                        }),
                        InputEvent::Keyboard { key } => events_stream.push(Event {
                            target: index,
                            event_type: EventType::KeyboardInput { key: *key },
                            ..Event::default()
                        }),
                    }
                }

                // Mouse is currently pressed for this node
                if self.is_mouse_pressed
                    && Self::contains_event(&self.previous_events, &index, &EventType::MousePressed)
                {
                    let mouse_pressed_event = Event {
                        target: index,
                        event_type: EventType::MousePressed,
                        ..Event::default()
                    };
                    events_stream.push(mouse_pressed_event);

                    // Make sure this event isn't removed while mouse is still held down
                    Self::insert_event(&mut next_events, &index, EventType::MousePressed);
                }
            }
        }

        if was_mouse_down_event && !was_focus_event && self.current_focus != Index::default() {
            let focus_event = Event {
                target: self.current_focus,
                event_type: EventType::Blur,
                ..Event::default()
            };
            events_stream.push(focus_event);
            self.current_focus = Index::default();
        }

        if was_mouse_down_event && was_focus_event && self.current_focus != self.last_focus {
            let focus_event = Event {
                target: self.last_focus,
                event_type: EventType::Blur,
                ..Event::default()
            };
            events_stream.push(focus_event);
        }

        // Replace the previous events with the next set
        self.previous_events = next_events;

        // Propagate Events
        for event in events_stream.iter_mut() {
            let mut parents: Vec<Index> = Vec::new();
            self.get_all_parents(event.target, &mut parents);

            // First call target
            let mut target_widget = self.widget_manager.take(event.target);
            target_widget.on_event(self, event);
            self.widget_manager.repossess(target_widget);

            // Event debugging
            // if matches!(event.event_type, EventType::Click) {
            //     dbg!("Click event!");
            //     let widget = self.widget_manager.take(event.target);
            //     dbg!(widget.get_name());
            //     self.widget_manager.repossess(widget);
            // }

            // TODO: Restore propagation.
            // for parent in parents {
            //     if event.should_propagate {
            //         let mut parent_widget = self.widget_manager.take(parent);
            //         parent_widget.on_event(self, event);
            //         self.widget_manager.repossess(parent_widget);
            //     }
            // }
        }
    }

    /// Insert an event for a widget in the given event map
    fn insert_event(
        events: &mut HashMap<Index, HashSet<EventType>>,
        widget_id: &Index,
        event_type: EventType,
    ) -> bool {
        let entry = events.entry(*widget_id).or_insert(HashSet::default());
        entry.insert(event_type)
    }

    /// Remove an event from a widget in the given event map
    #[allow(dead_code)]
    fn remove_event(
        events: &mut HashMap<Index, HashSet<EventType>>,
        widget_id: &Index,
        event_type: &EventType,
    ) -> bool {
        let entry = events.entry(*widget_id).or_insert(HashSet::default());
        entry.remove(event_type)
    }

    /// Checks if the given event map contains a specific event for the given widget
    fn contains_event(
        events: &HashMap<Index, HashSet<EventType>>,
        widget_id: &Index,
        event_type: &EventType,
    ) -> bool {
        if let Some(entry) = events.get(widget_id) {
            entry.contains(event_type)
        } else {
            false
        }
    }

    /// Checks if the given event map contains any events for the given widget
    #[allow(dead_code)]
    fn has_any_event(events: &HashMap<Index, HashSet<EventType>>, widget_id: &Index) -> bool {
        if let Some(entry) = events.get(widget_id) {
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
