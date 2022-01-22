use crate::flo_binding::{Binding, MutableBound};

use crate::cursor::CursorEvent;
use crate::layout_cache::Rect;
use crate::render_command::RenderCommand;
use crate::widget_manager::WidgetManager;
use crate::{
    Event, EventType, Index, InputEvent, InputEventCategory, KayakContext, KeyCode, KeyboardEvent,
    KeyboardModifiers, PointerEvents, Widget,
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

#[derive(Debug, Clone)]
pub(crate) struct EventDispatcher {
    is_mouse_pressed: bool,
    next_mouse_pressed: bool,
    current_mouse_position: (f32, f32),
    next_mouse_position: (f32, f32),
    previous_events: EventMap,
    keyboard_modifiers: KeyboardModifiers,
    pub last_clicked: Binding<Index>,
    contains_cursor: Option<bool>,
    wants_cursor: Option<bool>,
    has_cursor: Option<Index>,
    pub cursor_capture: Option<Index>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            last_clicked: Binding::new(Index::default()),
            is_mouse_pressed: Default::default(),
            next_mouse_pressed: Default::default(),
            current_mouse_position: Default::default(),
            next_mouse_position: Default::default(),
            previous_events: Default::default(),
            keyboard_modifiers: Default::default(),
            contains_cursor: None,
            wants_cursor: None,
            has_cursor: None,
            cursor_capture: None,
        }
    }

    /// Returns whether the mouse is currently pressed or not
    #[allow(dead_code)]
    pub fn is_mouse_pressed(&self) -> bool {
        self.is_mouse_pressed
    }

    /// Gets the current mouse position (since last mouse event)
    #[allow(dead_code)]
    pub fn current_mouse_position(&self) -> (f32, f32) {
        self.current_mouse_position
    }

    /// Captures all cursor events and instead makes the given index the target
    pub fn capture_cursor(&mut self, index: Index) -> Option<Index> {
        let old = self.cursor_capture;
        self.cursor_capture = Some(index);
        old
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
        if self.cursor_capture == Some(index) {
            self.force_release_cursor();
            true
        } else {
            false
        }
    }

    /// Releases the captured cursor
    ///
    /// Returns the index of the previous captor.
    ///
    /// This will force the release, regardless of which widget has called it. To safely release,
    /// use the standard [`release_cursor`](Self::release_cursor) method instead.
    pub fn force_release_cursor(&mut self) -> Option<Index> {
        let old = self.cursor_capture;
        self.cursor_capture = None;
        old
    }

    /// Returns true if the cursor is currently over a valid widget
    ///
    /// For the purposes of this method, a valid widget is one which has the means to display a visual component on its own.
    /// This means widgets specified with [`RenderCommand::Empty`], [`RenderCommand::Layout`], or [`RenderCommand::Clip`]
    /// do not meet the requirements to "contain" the cursor.
    #[allow(dead_code)]
    pub fn contains_cursor(&self) -> bool {
        self.contains_cursor.unwrap_or_default()
    }

    /// Returns true if the cursor may be needed by a widget or it's already in use by one
    ///
    /// This is useful for checking if certain events (such as a click) would "matter" to the UI at all. Example widgets
    /// include buttons, sliders, and text boxes.
    #[allow(dead_code)]
    pub fn wants_cursor(&self) -> bool {
        self.wants_cursor.unwrap_or_default() || self.has_cursor.is_some()
    }

    /// Returns true if the cursor is currently in use by a widget
    ///
    /// This is most often useful for checking drag events as it will still return true even if the drag continues outside
    /// the widget bounds (as long as it started within it).
    #[allow(dead_code)]
    pub fn has_cursor(&self) -> bool {
        self.has_cursor.is_some()
    }

    /// Process and dispatch an [InputEvent](crate::InputEvent)
    #[allow(dead_code)]
    pub fn process_event(&mut self, input_event: InputEvent, context: &mut KayakContext) {
        let events = self.build_event_stream(&[input_event], &mut context.widget_manager);
        self.dispatch_events(events, context);
    }

    /// Process and dispatch a set of [InputEvents](crate::InputEvent)
    pub fn process_events(&mut self, input_events: Vec<InputEvent>, context: &mut KayakContext) {
        let events = self.build_event_stream(&input_events, &mut context.widget_manager);
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
        for mut event in events {
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

                event.default_prevented |= node_event.default_prevented;

                // --- Propagate Event --- //
                if node_event.should_propagate {
                    current_target = context.widget_manager.node_tree.get_parent(index);
                } else {
                    current_target = None;
                }
            }

            if !event.default_prevented {
                self.execute_default(event, context);
            }
        }

        // === Maintain Events === //
        // Events that need to be maintained without re-firing between event updates should be managed here
        for (index, events) in &self.previous_events {
            // Mouse is currently pressed for this node
            if self.is_mouse_pressed && events.contains(&EventType::MouseDown(Default::default())) {
                // Make sure this event isn't removed while mouse is still held down
                Self::insert_event(
                    &mut next_events,
                    index,
                    EventType::MouseDown(Default::default()),
                );
            }

            // Mouse is currently within this node
            if events.contains(&EventType::MouseIn(Default::default()))
                && !Self::contains_event(
                    &next_events,
                    index,
                    &EventType::MouseOut(Default::default()),
                )
            {
                // Make sure this event isn't removed while mouse is still within node
                Self::insert_event(
                    &mut next_events,
                    index,
                    EventType::MouseIn(Default::default()),
                );
            }
        }

        // Replace the previous events with the next set
        self.previous_events = next_events;
    }

    /// Generates a stream of [Events](crate::Event) from a set of [InputEvents](crate::InputEvent)
    fn build_event_stream(
        &mut self,
        input_events: &[InputEvent],
        widget_manager: &mut WidgetManager,
    ) -> Vec<Event> {
        let mut event_stream = Vec::<Event>::new();
        let mut states: HashMap<EventType, EventState> = HashMap::new();

        let root = if let Some(root) = widget_manager.node_tree.root_node {
            root
        } else {
            return event_stream;
        };

        // === Setup Cursor States === //
        let old_contains_cursor = self.contains_cursor;
        let old_wants_cursor = self.wants_cursor;
        self.contains_cursor = None;
        self.wants_cursor = None;
        self.next_mouse_position = self.current_mouse_position;
        self.next_mouse_pressed = self.is_mouse_pressed;

        // --- Pre-Process --- //
        // We pre-process some events so that we can provide accurate event data (such as if the mouse is pressed)
        // This is faster than resolving data after the fact since `input_events` is generally very small
        for input_event in input_events {
            if let InputEvent::MouseMoved(point) = input_event {
                // Reset next global mouse position
                self.next_mouse_position = *point;
            }

            if matches!(input_event, InputEvent::MouseLeftPress) {
                // Reset next global mouse pressed
                self.next_mouse_pressed = true;
                break;
            } else if matches!(input_event, InputEvent::MouseLeftRelease) {
                // Reset next global mouse pressed
                self.next_mouse_pressed = false;
                // Reset global cursor container
                self.has_cursor = None;
                break;
            }
        }

        // === Mouse Events === //
        if let Some(captor) = self.cursor_capture {
            // A widget has been set to capture pointer events -> it should be the only one receiving events
            for input_event in input_events {
                // --- Process Event --- //
                if matches!(input_event.category(), InputEventCategory::Mouse) {
                    // A widget's PointerEvents style will determine how it and its children are processed
                    let pointer_events = Self::resolve_pointer_events(captor, widget_manager);

                    match pointer_events {
                        PointerEvents::All | PointerEvents::SelfOnly => {
                            let events = self.process_pointer_events(
                                input_event,
                                (captor, 0),
                                &mut states,
                                widget_manager,
                                true,
                            );
                            event_stream.extend(events);
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // No capturing widget -> process cursor events as normal
            let mut stack: Vec<TreeNode> = vec![(root, 0)];
            while stack.len() > 0 {
                let (current, depth) = stack.pop().unwrap();
                let mut enter_children = true;

                for input_event in input_events {
                    // --- Process Event --- //
                    if matches!(input_event.category(), InputEventCategory::Mouse) {
                        // A widget's PointerEvents style will determine how it and its children are processed
                        let pointer_events = Self::resolve_pointer_events(current, widget_manager);

                        match pointer_events {
                            PointerEvents::All | PointerEvents::SelfOnly => {
                                let events = self.process_pointer_events(
                                    input_event,
                                    (current, depth),
                                    &mut states,
                                    widget_manager,
                                    false,
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
                        if let Some(current_focus) = widget_manager.focus_tree.current() {
                            if current_focus != node {
                                event_stream.push(Event::new(current_focus, EventType::Blur));
                            }
                        }
                        widget_manager.focus_tree.focus(node);
                    }
                    _ => {}
                }
            }
        }

        // --- Blur Event --- //
        if !had_focus_event && input_events.contains(&InputEvent::MouseLeftPress) {
            // A mouse press didn't contain a focus event -> blur
            if let Some(current_focus) = widget_manager.focus_tree.current() {
                event_stream.push(Event::new(current_focus, EventType::Blur));
                widget_manager.focus_tree.blur();
            }
        }

        // === Process Cursor States === //
        self.current_mouse_position = self.next_mouse_position;
        self.is_mouse_pressed = self.next_mouse_pressed;

        if self.contains_cursor.is_none() {
            // No change -> revert
            self.contains_cursor = old_contains_cursor;
        }
        if self.wants_cursor.is_none() {
            // No change -> revert
            self.wants_cursor = old_wants_cursor;
        }

        event_stream
    }

    fn process_pointer_events(
        &mut self,
        input_event: &InputEvent,
        tree_node: TreeNode,
        states: &mut HashMap<EventType, EventState>,
        widget_manager: &WidgetManager,
        ignore_layout: bool,
    ) -> Vec<Event> {
        let mut event_stream = Vec::<Event>::new();
        let (node, depth) = tree_node;

        match input_event {
            InputEvent::MouseMoved(point) => {
                if let Some(layout) = widget_manager.get_layout(&node) {
                    let cursor_event = self.get_cursor_event(*point);
                    let was_contained = layout.contains(&self.current_mouse_position);
                    let is_contained = layout.contains(point);
                    if !ignore_layout && was_contained != is_contained {
                        if was_contained {
                            event_stream.push(Event::new(node, EventType::MouseOut(cursor_event)));
                        } else {
                            event_stream.push(Event::new(node, EventType::MouseIn(cursor_event)));
                        }
                    }
                    if self.contains_cursor.is_none() || !self.contains_cursor.unwrap_or_default() {
                        if let Some(widget) = widget_manager.current_widgets.get(node).unwrap() {
                            // Check if the cursor moved onto a widget that qualifies as one that can contain it
                            if ignore_layout || Self::can_contain_cursor(widget) {
                                self.contains_cursor = Some(is_contained);
                            }
                        }
                    }

                    if self.wants_cursor.is_none() || !self.wants_cursor.unwrap_or_default() {
                        let focusable = widget_manager.get_focusable(node);
                        // Check if the cursor moved onto a focusable widget (i.e. one that would want it)
                        if matches!(focusable, Some(true)) {
                            self.wants_cursor = Some(is_contained);
                        }
                    }

                    // Check for hover eligibility
                    if ignore_layout || is_contained {
                        Self::update_state(
                            states,
                            (node, depth),
                            layout,
                            EventType::Hover(cursor_event),
                        );
                    }
                }
            }
            InputEvent::MouseLeftPress => {
                if let Some(layout) = widget_manager.get_layout(&node) {
                    if ignore_layout || layout.contains(&self.current_mouse_position) {
                        let cursor_event = self.get_cursor_event(self.current_mouse_position);
                        event_stream.push(Event::new(node, EventType::MouseDown(cursor_event)));

                        if let Some(focusable) = widget_manager.get_focusable(node) {
                            if focusable {
                                Self::update_state(states, (node, depth), layout, EventType::Focus);
                            }
                        }

                        if self.has_cursor.is_none() {
                            let widget = widget_manager.current_widgets.get(node).unwrap();
                            if let Some(widget) = widget {
                                // Check if the cursor moved onto a widget that qualifies as one that can contain it
                                if Self::can_contain_cursor(widget) {
                                    self.has_cursor = Some(node);
                                }
                            }
                        }
                    }
                }
            }
            InputEvent::MouseLeftRelease => {
                if let Some(layout) = widget_manager.get_layout(&node) {
                    if ignore_layout || layout.contains(&self.current_mouse_position) {
                        let cursor_event = self.get_cursor_event(self.current_mouse_position);
                        event_stream.push(Event::new(node, EventType::MouseUp(cursor_event)));
                        self.last_clicked.set(node);

                        if Self::contains_event(
                            &self.previous_events,
                            &node,
                            &EventType::MouseDown(cursor_event),
                        ) {
                            Self::update_state(
                                states,
                                (node, depth),
                                layout,
                                EventType::Click(cursor_event),
                            );
                        }
                    }
                }
            }
            _ => {}
        }

        event_stream
    }

    fn resolve_pointer_events(index: Index, widget_manager: &WidgetManager) -> PointerEvents {
        let mut pointer_events = PointerEvents::default();
        if let Some(widget) = widget_manager.current_widgets.get(index).unwrap() {
            if let Some(styles) = widget.get_styles() {
                pointer_events = styles.pointer_events.resolve();
            }
        }
        pointer_events
    }

    fn get_cursor_event(&self, position: (f32, f32)) -> CursorEvent {
        let change = self.next_mouse_pressed != self.is_mouse_pressed;
        let pressed = self.next_mouse_pressed;
        CursorEvent {
            position,
            pressed,
            just_pressed: change && pressed,
            just_released: change && !pressed,
        }
    }

    fn process_keyboard_events(
        &mut self,
        input_event: &InputEvent,
        _states: &mut HashMap<EventType, EventState>,
        widget_manager: &WidgetManager,
    ) -> Vec<Event> {
        let mut event_stream = Vec::new();
        if let Some(current_focus) = widget_manager.focus_tree.current() {
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

    /// Checks if the given widget is eligible to "contain" the cursor (i.e. the cursor is considered contained when hovering over it)
    ///
    /// Currently a valid widget is defined as one where:
    /// * RenderCommands is neither `Empty` nor `Layout` nor `Clip`
    fn can_contain_cursor(widget: &Box<dyn Widget>) -> bool {
        if let Some(styles) = widget.get_styles() {
            let cmds = styles.render_command.resolve();
            !matches!(
                cmds,
                RenderCommand::Empty | RenderCommand::Layout | RenderCommand::Clip
            )
        } else {
            false
        }
    }

    /// Executes default actions for events
    fn execute_default(&mut self, event: Event, context: &mut KayakContext) {
        match event.event_type {
            EventType::KeyDown(evt) => match evt.key() {
                KeyCode::Tab => {
                    let current_focus = context.widget_manager.focus_tree.current();

                    let index = if evt.is_shift_pressed() {
                        context.widget_manager.focus_tree.prev()
                    } else {
                        context.widget_manager.focus_tree.next()
                    };

                    if let Some(index) = index {
                        let mut events = vec![Event::new(index, EventType::Focus)];
                        if let Some(current_focus) = current_focus {
                            if current_focus != index {
                                events.push(Event::new(current_focus, EventType::Blur));
                            }
                        }
                        context.widget_manager.focus_tree.focus(index);
                        self.dispatch_events(events, context);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// Merge this `EventDispatcher` with another, taking only the internally mutated data.
    ///
    /// This is meant to solve the issue in `KayakContext`, where [`EventDispatcher::process_events`] and
    /// similar methods require mutable access to `KayakContext`, forcing `EventDispatcher` to be cloned
    /// before running the method. However, some data mutated through `KayakContext` may be lost when
    /// re-claiming the `EventDispatcher`. This method ensures that data mutated in such a way will not be
    /// overwritten during the merge.
    ///
    /// # Arguments
    ///
    /// * `from`: The other `EventDispatcher` to merge from
    ///
    /// returns: ()
    pub fn merge(&mut self, from: EventDispatcher) {
        // Merge only what could be changed internally. External changes (i.e. from KayakContext)
        // should not be touched
        self.last_clicked = from.last_clicked;
        self.is_mouse_pressed = from.is_mouse_pressed;
        self.next_mouse_pressed = from.next_mouse_pressed;
        self.current_mouse_position = from.current_mouse_position;
        self.next_mouse_position = from.next_mouse_position;
        self.previous_events = from.previous_events;
        self.keyboard_modifiers = from.keyboard_modifiers;
        self.contains_cursor = from.contains_cursor;
        self.wants_cursor = from.wants_cursor;
        self.has_cursor = from.has_cursor;

        // Do not include:
        // self.cursor_capture = from.cursor_capture;
    }
}
