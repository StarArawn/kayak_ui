use flo_binding::{Binding, MutableBound};

use crate::layout_cache::Rect;
use crate::widget_manager::WidgetManager;
use crate::{
    Event, EventType, Index, InputEvent, InputEventCategory, KayakContext, KeyCode, KeyboardEvent,
    KeyboardModifiers, PointerEvents,
};
use std::collections::{HashMap, HashSet};

type EventMap = HashMap<Index, HashSet<EventType>>;
type TreeNode = (
    // The node ID
    Index,
    // The node depth
    isize,
);

#[derive(Debug, Clone)]
struct EventState {
    best_z_index: f32,
    best_match: Option<Index>,
    best_depth: isize,
}

impl Default for EventState {
    fn default() -> Self {
        Self {
            best_z_index: f32::NEG_INFINITY,
            best_match: None,
            best_depth: -1,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct EventDispatcher {
    is_mouse_pressed: bool,
    current_focus: Option<Index>,
    current_mouse_position: (f32, f32),
    next_mouse_position: (f32, f32),
    previous_events: EventMap,
    keyboard_modifiers: KeyboardModifiers,
    pub last_clicked: Binding<Index>,
}

impl EventDispatcher {
    /// Returns whether the mouse is currently pressed or not
    #[allow(dead_code)]
    pub fn is_mouse_pressed(&self) -> bool {
        self.is_mouse_pressed
    }

    /// Gets the currently focused widget
    #[allow(dead_code)]
    pub fn current_focus(&self) -> Option<Index> {
        self.current_focus
    }

    /// Gets the current mouse position (since last mouse event)
    #[allow(dead_code)]
    pub fn current_mouse_position(&self) -> (f32, f32) {
        self.current_mouse_position
    }

    /// Process and dispatch an [InputEvent](crate::InputEvent)
    #[allow(dead_code)]
    pub fn process_event(&mut self, input_event: InputEvent, context: &mut KayakContext) {
        let events = self.build_event_stream(&[input_event], &context.widget_manager);
        self.dispatch_events(events, context);
    }

    /// Process and dispatch a set of [InputEvents](crate::InputEvent)
    pub fn process_events(&mut self, input_events: Vec<InputEvent>, context: &mut KayakContext) {
        let events = self.build_event_stream(&input_events, &context.widget_manager);
        self.dispatch_events(events, context);
    }

    /// Dispatch an [Event](crate::Event)
    #[allow(dead_code)]
    pub fn dispatch_event(&mut self, event: Event, context: &mut KayakContext) {
        self.dispatch_events(vec![event], context);
    }

    /// Dispatch a set of [Events](crate::Event)
    pub fn dispatch_events(&mut self, events: Vec<Event>, context: &mut KayakContext) {
        // === Dispatch Events === //
        let mut next_events = HashMap::default();
        for event in events {
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
                Self::insert_event(&mut next_events, &index, node_event.event_type);

                // --- Call Event --- //
                let mut target_widget = context.widget_manager.take(index);
                target_widget.on_event(context, &mut node_event);
                context.widget_manager.repossess(target_widget);

                // --- Propagate Event --- //
                if node_event.should_propagate {
                    current_target = context.widget_manager.node_tree.get_parent(index);
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
                && !Self::contains_event(&next_events, index, &EventType::MouseOut)
            {
                // Make sure this event isn't removed while mouse is still within node
                Self::insert_event(&mut next_events, index, EventType::MouseIn);
            }
        }

        // Replace the previous events with the next set
        self.previous_events = next_events;
    }

    /// Generates a stream of [Events](crate::Event) from a set of [InputEvents](crate::InputEvent)
    fn build_event_stream(
        &mut self,
        input_events: &[InputEvent],
        widget_manager: &WidgetManager,
    ) -> Vec<Event> {
        let mut event_stream = Vec::<Event>::new();
        let mut states: HashMap<EventType, EventState> = HashMap::new();

        let root = if let Some(root) = widget_manager.node_tree.root_node {
            root
        } else {
            return event_stream;
        };

        // === Mouse Events === //
        let mut stack: Vec<TreeNode> = vec![(root, 0)];
        while stack.len() > 0 {
            let (current, depth) = stack.pop().unwrap();
            let mut enter_children = true;

            for input_event in input_events {
                // --- Process Event --- //
                if matches!(input_event.category(), InputEventCategory::Mouse) {
                    // A widget's PointerEvents style will determine how it and its children are processed
                    let mut pointer_events = PointerEvents::default();
                    if let Some(widget) = widget_manager.current_widgets.get(current).unwrap() {
                        if let Some(styles) = widget.get_styles() {
                            pointer_events = styles.pointer_events.resolve();
                        }
                    }

                    match pointer_events {
                        PointerEvents::All | PointerEvents::SelfOnly => {
                            let events = self.process_pointer_events(
                                input_event,
                                (current, depth),
                                &mut states,
                                widget_manager,
                            );
                            event_stream.extend(events);

                            if matches!(pointer_events, PointerEvents::SelfOnly) {
                                enter_children = false;
                            }
                        }
                        PointerEvents::None => enter_children = false,
                        PointerEvents::ChildrenOnly => {}
                    }
                }
            }

            // --- Push Children to Stack --- //
            if enter_children {
                if let Some(children) = widget_manager.node_tree.children.get(&current) {
                    for child in children {
                        stack.push((*child, depth + 1));
                    }
                }
            }
        }

        // === Keyboard Events === //
        for input_event in input_events {
            // Keyboard events only care about the currently focused widget so we don't need to run this over every node in the tree
            let events = self.process_keyboard_events(input_event, &mut states, widget_manager);
            event_stream.extend(events);
        }

        // === Additional Events === //
        let mut had_focus_event = false;

        // These events are ones that require a specific target and need the tree to be evaluated before selecting the best match
        for (event_type, state) in states {
            if let Some(node) = state.best_match {
                event_stream.push(Event::new(node, event_type));

                match event_type {
                    EventType::Focus => {
                        had_focus_event = true;
                        if let Some(current_focus) = self.current_focus {
                            if current_focus != node {
                                event_stream.push(Event::new(current_focus, EventType::Blur));
                            }
                        }
                        self.current_focus = Some(node);
                    }
                    _ => {}
                }
            }
        }

        // --- Blur Event --- //
        if !had_focus_event && input_events.contains(&InputEvent::MouseLeftPress) {
            // A mouse press didn't contain a focus event -> blur
            if let Some(current_focus) = self.current_focus {
                event_stream.push(Event::new(current_focus, EventType::Blur));
                self.current_focus = None;
            }
        }

        // Apply changes
        self.current_mouse_position = self.next_mouse_position;

        event_stream
    }

    fn process_pointer_events(
        &mut self,
        input_event: &InputEvent,
        tree_node: TreeNode,
        states: &mut HashMap<EventType, EventState>,
        widget_manager: &WidgetManager,
    ) -> Vec<Event> {
        let mut event_stream = Vec::<Event>::new();
        let (node, depth) = tree_node;

        match input_event {
            InputEvent::MouseMoved(point) => {
                if let Some(layout) = widget_manager.get_layout(&node) {
                    let was_contained = layout.contains(&self.current_mouse_position);
                    let is_contained = layout.contains(point);
                    if was_contained != is_contained {
                        if was_contained {
                            event_stream.push(Event::new(node, EventType::MouseOut));
                        } else {
                            event_stream.push(Event::new(node, EventType::MouseIn));
                        }
                    }

                    // Check for hover eligibility
                    if is_contained {
                        Self::update_state(states, (node, depth), layout, EventType::Hover);
                    }
                }

                // Reset global mouse position
                self.next_mouse_position = *point;
            }
            InputEvent::MouseLeftPress => {
                // Reset global mouse pressed
                self.is_mouse_pressed = true;

                if let Some(layout) = widget_manager.get_layout(&node) {
                    if layout.contains(&self.current_mouse_position) {
                        event_stream.push(Event::new(node, EventType::MouseDown));

                        if let Some(widget) = widget_manager.current_widgets.get(node).unwrap() {
                            if widget.focusable() {
                                Self::update_state(states, (node, depth), layout, EventType::Focus);
                            }
                        }
                    }
                }
            }
            InputEvent::MouseLeftRelease => {
                // Reset global mouse pressed
                self.is_mouse_pressed = false;

                if let Some(layout) = widget_manager.get_layout(&node) {
                    if layout.contains(&self.current_mouse_position) {
                        event_stream.push(Event::new(node, EventType::MouseUp));
                        self.last_clicked.set(node);

                        if Self::contains_event(&self.previous_events, &node, &EventType::MouseDown)
                        {
                            Self::update_state(states, (node, depth), layout, EventType::Click);
                        }
                    }
                }
            }
            _ => {}
        }

        event_stream
    }

    fn process_keyboard_events(
        &mut self,
        input_event: &InputEvent,
        _states: &mut HashMap<EventType, EventState>,
        _widget_manager: &WidgetManager,
    ) -> Vec<Event> {
        let mut event_stream = Vec::new();
        if let Some(current_focus) = self.current_focus {
            match input_event {
                InputEvent::CharEvent { c } => {
                    event_stream.push(Event::new(current_focus, EventType::CharInput { c: *c }))
                }
                InputEvent::Keyboard { key, is_pressed } => {
                    // === Modifers === //
                    match key {
                        KeyCode::LControl | KeyCode::RControl => {
                            self.keyboard_modifiers.is_ctrl_pressed = *is_pressed
                        }
                        KeyCode::LShift | KeyCode::RShift => {
                            self.keyboard_modifiers.is_shift_pressed = *is_pressed
                        }
                        KeyCode::LAlt | KeyCode::RAlt => {
                            self.keyboard_modifiers.is_alt_pressed = *is_pressed
                        }
                        KeyCode::LWin | KeyCode::RWin => {
                            self.keyboard_modifiers.is_meta_pressed = *is_pressed
                        }
                        _ => {}
                    }

                    // === Event === //
                    if *is_pressed {
                        event_stream.push(Event::new(
                            current_focus,
                            EventType::KeyDown(KeyboardEvent::new(*key, self.keyboard_modifiers)),
                        ))
                    } else {
                        event_stream.push(Event::new(
                            current_focus,
                            EventType::KeyUp(KeyboardEvent::new(*key, self.keyboard_modifiers)),
                        ))
                    }
                }
                _ => {}
            }
        }

        event_stream
    }

    /// Updates the state data for the given event
    fn update_state(
        states: &mut HashMap<EventType, EventState>,
        tree_node: TreeNode,
        layout: &Rect,
        event_type: EventType,
    ) {
        let state = states.entry(event_type).or_insert(EventState::default());

        let (node, depth) = tree_node;
        // Node is at or above best depth and is at or above best z-level
        let mut should_update = depth >= state.best_depth && layout.z_index >= state.best_z_index;
        // OR node is above best z-level
        should_update |= layout.z_index > state.best_z_index;

        if should_update {
            state.best_match = Some(node);
            state.best_z_index = layout.z_index;
            state.best_depth = depth;
        }
    }

    /// Checks if the given event map contains a specific event for the given widget
    fn contains_event(events: &EventMap, widget_id: &Index, event_type: &EventType) -> bool {
        if let Some(entry) = events.get(widget_id) {
            entry.contains(event_type)
        } else {
            false
        }
    }

    /// Insert an event for a widget in the given event map
    fn insert_event(events: &mut EventMap, widget_id: &Index, event_type: EventType) -> bool {
        let entry = events.entry(*widget_id).or_insert(HashSet::default());
        entry.insert(event_type)
    }
}
