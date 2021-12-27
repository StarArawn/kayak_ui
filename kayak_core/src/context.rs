use flo_binding::Changeable;
use morphorm::Hierarchy;
use std::collections::{HashMap, HashSet};

use crate::{
    multi_state::MultiState, widget_manager::WidgetManager, Event, EventType, Index, InputEvent, InputEventCategory,
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
        let mut event_stream = Vec::<Event>::new();

        // === Find Event Targets === //
        for input_event in input_events.iter() {
            match input_event.category() {
                InputEventCategory::Mouse => {
                    event_stream.extend(self.process_pointer_events(input_event));
                }
                InputEventCategory::Keyboard => {
                    event_stream.extend(self.process_keyboard_events(input_event));
                }
            }
        }

        // === Process Events === //
        let mut next_events = HashMap::default();
        for event in event_stream {
            let mut current_target: Option<Index> = Some(event.target);
            while let Some(index) = current_target {
                // Create a copy of the event, specific for this node
                // This is to make sure unauthorized changes to the event are not propagated
                // (e.g., changing the event type, removing the target, etc.)
                let mut node_event = Event {
                    current_target: index,
                    ..event
                };

                // --- Update State --- //
                Self::insert_event(
                    &mut next_events,
                    &index,
                    node_event.event_type,
                );

                // --- Call Event --- //
                let mut target_widget = self.widget_manager.take(index);
                target_widget.on_event(self, &mut node_event);
                self.widget_manager.repossess(target_widget);

                // --- Propagate Event --- //
                if node_event.should_propagate {
                    current_target = self.widget_manager.node_tree.get_parent(index);
                } else {
                    current_target = None;
                }
            }
        }

        // === Maintain Events === //
        // Events that need to be maintained without re-firing between event updates should be managed here
        for (index, events) in &self.previous_events {
            // Mouse is currently pressed for this node
            if self.is_mouse_pressed && events.contains(&EventType::MouseDown) {
                // Make sure this event isn't removed while mouse is still held down
                Self::insert_event(&mut next_events, index, EventType::MouseDown);
            }

            // Mouse is currently within this node
            if events.contains(&EventType::MouseIn)
                && !Self::contains_event(&next_events, index, &EventType::MouseOut) {
                // Make sure this event isn't removed while mouse is still within node
                Self::insert_event(&mut next_events, index, EventType::MouseIn);
            }
        }

        // Replace the previous events with the next set
        self.previous_events = next_events;
    }

    fn process_pointer_events(&mut self, input_event: &InputEvent) -> Vec<Event> {
        let mut event_stream = Vec::new();

        match input_event {
            InputEvent::MouseMoved(point) => {
                if let Some((next, next_nodes)) = self.widget_manager.get_nodes_under(*point, None) {
                    event_stream.push(Event::new(next, EventType::Hover));

                    // Mouse In - Applies to all matching nodes
                    for next in next_nodes {
                        if let Some(rect) = self.widget_manager.layout_cache.rect.get(&next) {
                            if !rect.contains(&self.last_mouse_position) {
                                event_stream.push(Event::new(next, EventType::MouseIn));
                            }
                        }
                    }
                }

                if let Some((.., prev_nodes)) = self.widget_manager.get_nodes_under(self.last_mouse_position, None) {
                    // Mouse Out - Applies to all matching nodes
                    for prev in prev_nodes {
                        if let Some(rect) = self.widget_manager.layout_cache.rect.get(&prev) {
                            if !rect.contains(point) {
                                event_stream.push(Event::new(prev, EventType::MouseOut));
                            }
                        }
                    }
                }

                // Reset global mouse position
                self.last_mouse_position = *point;
            }
            InputEvent::MouseLeftPress => {
                // Reset global mouse pressed
                self.is_mouse_pressed = true;

                if let Some((prev, ..)) = self.widget_manager.get_nodes_under(self.last_mouse_position, None) {
                    event_stream.push(Event::new(prev, EventType::MouseDown));

                    // Find a focusable widget in the hierarchy
                    let mut next_focus: Option<Index> = None;
                    let mut index: Option<Index> = Some(prev);
                    while let Some(idx) = index {
                        index = None;
                        if let Some(widget) = self.widget_manager.current_widgets.get(idx).unwrap() {
                            if widget.focusable() {
                                next_focus = Some(idx);
                                event_stream.push(Event::new(idx, EventType::Focus));
                            } else {
                                index = self.widget_manager.node_tree.parent(idx);
                            }
                        }
                    }

                    if let Some(index) = next_focus {
                        // Was a focus event
                        if self.current_focus != index {
                            // New focus
                            event_stream.push(Event::new(self.current_focus, EventType::Blur));
                        }
                        // Update focus
                        self.current_focus = index;
                    } else if self.current_focus != Index::default() {
                        // Was a blur event
                        event_stream.push(Event::new(self.current_focus, EventType::Blur));
                    }
                }
            }
            InputEvent::MouseLeftRelease => {
                // Reset global mouse pressed
                self.is_mouse_pressed = false;

                if let Some((prev, ..)) = self.widget_manager.get_nodes_under(self.last_mouse_position, None) {
                    event_stream.push(Event::new(prev, EventType::MouseUp));

                    if Self::contains_event(
                        &self.previous_events,
                        &prev,
                        &EventType::MouseDown,
                    ) {
                        event_stream.push(Event::new(prev, EventType::Click));
                    }
                }
            }
            _ => {}
        }

        event_stream
    }

    fn process_keyboard_events(&mut self, input_event: &InputEvent) -> Vec<Event> {
        let mut event_stream = Vec::new();
        match input_event {
            InputEvent::CharEvent { c } => event_stream.push(
                Event::new(self.current_focus, EventType::CharInput { c: *c })
            ),
            InputEvent::Keyboard { key } => event_stream.push(
                Event::new(self.current_focus, EventType::KeyboardInput { key: *key })
            ),
            _ => {}
        }

        event_stream
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
