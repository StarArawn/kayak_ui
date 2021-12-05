use std::collections::HashMap;

use as_any::AsAny;
use resources::Ref;

use crate::{node::NodeIndex, widget_manager::WidgetManager, Event, EventType, Index, InputEvent};

pub trait GlobalState: Send + Sync {}

pub struct KayakContext<'a> {
    component_states: HashMap<crate::Index, resources::Resources>,
    current_id: crate::Index,
    pub widget_manager: WidgetManager,
    last_mouse_position: (f32, f32),
    global_state: Option<&'a mut dyn GlobalState>,
}

impl<'a> std::fmt::Debug for KayakContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KayakContext")
            .field("current_id", &self.current_id)
            .finish()
    }
}

impl<'a> KayakContext<'a> {
    pub fn new() -> Self {
        Self {
            component_states: HashMap::new(),
            current_id: crate::Index::default(),
            widget_manager: WidgetManager::new(),
            last_mouse_position: (0.0, 0.0),
            global_state: None,
        }
    }

    pub fn set_current_id(&mut self, id: crate::Index) {
        self.current_id = id;
    }

    pub fn create_state<T: resources::Resource + Clone>(
        &mut self,
        initial_state: T,
    ) -> Option<Ref<T>> {
        if self.component_states.contains_key(&self.current_id) {
            let states = self.component_states.get_mut(&self.current_id).unwrap();
            if !states.contains::<T>() {
                states.insert(initial_state);
            }
        } else {
            let mut states = resources::Resources::default();
            states.insert(initial_state);
            self.component_states.insert(self.current_id, states);
        }
        return self.get_state();
    }

    fn get_state<T: resources::Resource + Clone>(&self) -> Option<Ref<T>> {
        if self.component_states.contains_key(&self.current_id) {
            let states = self.component_states.get(&self.current_id).unwrap();
            if let Ok(state) = states.get::<T>() {
                return Some(state);
            }
        }
        return None;
    }

    pub fn set_state<T: resources::Resource + Clone>(&mut self, state: T) {
        if self.component_states.contains_key(&self.current_id) {
            let states = self.component_states.get(&self.current_id).unwrap();
            if states.contains::<T>() {
                let mut mutate_t = states.get_mut::<T>().unwrap();
                self.widget_manager.dirty_nodes.push(self.current_id);
                *mutate_t = state;
            } else {
                panic!(
                    "No specific state created for component with id: {:?}!",
                    self.current_id
                );
            }
        } else {
            // Do nothing..
        }
    }

    pub fn render(&mut self, global_state: &'a mut dyn GlobalState) {
        self.global_state = Some(global_state);

        let dirty_nodes = self.widget_manager.dirty_nodes.clone();
        for node_index in dirty_nodes {
            if self
                .widget_manager
                .dirty_nodes
                .iter()
                .any(|dirty_index| node_index == *dirty_index)
            {
                let mut widget = self.widget_manager.take(node_index);
                widget.render(self);
                self.widget_manager.repossess(widget);
            }
        }

        self.widget_manager.dirty_nodes.clear();

        self.widget_manager.render();
        self.widget_manager.calculate_layout();

        self.global_state = None;
    }

    pub fn process_events(&mut self, input_events: Vec<InputEvent>) {
        let mut events_stream = Vec::new();
        for (index, _) in self.widget_manager.nodes.iter() {
            if let Some(layout) = self.widget_manager.layout_cache.rect.get(&NodeIndex(index)) {
                for input_event in input_events.iter() {
                    match input_event {
                        InputEvent::MouseMoved(point) => {
                            // Hover event.
                            if layout.contains(point) {
                                let hover_event = Event {
                                    target: index,
                                    event_type: EventType::Hover,
                                    ..Event::default()
                                };
                                events_stream.push(hover_event);
                            }
                            self.last_mouse_position = *point;
                        }
                        InputEvent::MouseLeftClick => {
                            if layout.contains(&self.last_mouse_position) {
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

    fn get_all_parents(&self, current: Index, parents: &mut Vec<Index>) {
        if let Some(parent) = self.widget_manager.tree.parents.get(&current) {
            parents.push(*parent);
            self.get_all_parents(*parent, parents);
        }
    }
}
