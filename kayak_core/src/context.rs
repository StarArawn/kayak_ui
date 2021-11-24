use std::collections::HashMap;

use resources::Ref;

use crate::widget_manager::WidgetManager;

pub struct KayakContext {
    component_states: HashMap<crate::Index, resources::Resources>,
    current_id: crate::Index,
    pub widget_manager: WidgetManager,
}

impl std::fmt::Debug for KayakContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KayakContext")
            .field("current_id", &self.current_id)
            .finish()
    }
}

impl KayakContext {
    pub fn new() -> Self {
        Self {
            component_states: HashMap::new(),
            current_id: crate::Index::default(),
            widget_manager: WidgetManager::new(),
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
            panic!(
                "No state created for component with id: {:?}!",
                self.current_id
            );
        }
    }

    pub fn render(&mut self) {
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
    }
}
