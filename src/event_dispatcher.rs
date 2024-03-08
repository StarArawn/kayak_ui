use bevy::{
    prelude::{Component, Entity, KeyCode, Resource, World},
    utils::{HashMap, HashSet},
};

use crate::{
    context::KayakRootContext,
    cursor::{CursorEvent, PointerEvents, ScrollEvent, ScrollUnit},
    event::{EventType, KEvent},
    focus_tree::FocusTree,
    input_event::{InputEvent, InputEventCategory},
    keyboard_event::{KeyboardEvent, KeyboardModifiers},
    layout::Rect,
    node::{Node, WrappedIndex},
    on_event::OnEvent,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle, RenderCommand},
    Focusable,
};

type EventMap = HashMap<WrappedIndex, HashSet<EventType>>;
type TreeNode = (
    // The node ID
    WrappedIndex,
    // The node depth
    isize,
);

#[derive(Debug, Clone)]
struct EventState {
    best_z_index: f32,
    best_match: Option<WrappedIndex>,
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

#[derive(Component, Debug, Clone, Default)]
pub struct EventDispatcher {
    is_mouse_pressed: bool,
    next_mouse_pressed: bool,
    current_mouse_position: (f32, f32),
    next_mouse_position: (f32, f32),
    previous_events: EventMap,
    keyboard_modifiers: KeyboardModifiers,
    // pub last_clicked: Binding<WrappedIndex>,
    contains_cursor: Option<bool>,
    wants_cursor: Option<bool>,
    has_cursor: Option<WrappedIndex>,
    pub(crate) cursor_capture: Option<WrappedIndex>,
    pub(crate) hovered: Option<WrappedIndex>,
}

impl EventDispatcher {
    pub(crate) fn new() -> Self {
        Self {
            // last_clicked: Binding::new(WrappedIndex(Entity::from_raw(0))),
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
            hovered: None,
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
    pub fn capture_cursor(&mut self, index: Entity) -> Option<WrappedIndex> {
        let old = self.cursor_capture;
        self.cursor_capture = Some(WrappedIndex(index));
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
    pub fn release_cursor(&mut self, index: Entity) -> bool {
        if self.cursor_capture == Some(WrappedIndex(index)) {
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
    pub fn force_release_cursor(&mut self) -> Option<WrappedIndex> {
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

    /// The currently hovered node
    #[allow(dead_code)]
    pub fn hovered(&self) -> Option<WrappedIndex> {
        self.hovered
    }

    /// Process and dispatch an [InputEvent](crate::InputEvent)
    // #[allow(dead_code)]
    // pub fn process_event(
    //     &mut self,
    //     input_event: InputEvent,
    //     context: &mut Context,
    //     focusable_query: &Query<&Focusable>,
    //     style_query: &Query<&Style>,
    //     on_event_query: &Query<&OnEvent>,
    // ) {
    //     let events = self.build_event_stream(&[input_event], context, focusable_query, style_query);
    //     self.dispatch_events(events, context, on_event_query);
    // }

    /// Process and dispatch a set of [InputEvents](crate::InputEvent)
    pub(crate) fn process_events(
        &mut self,
        input_events: &[InputEvent],
        context: &mut KayakRootContext,
        world: &mut World,
    ) {
        let events = { self.build_event_stream(input_events, context, world) };
        self.dispatch_events(events, context, world);
    }

    /// Dispatch an [KEvent](crate::KEvent)
    #[allow(dead_code)]
    pub fn dispatch_event(
        &mut self,
        event: KEvent,
        context: &mut KayakRootContext,
        world: &mut World,
    ) {
        self.dispatch_events(vec![event], context, world);
    }

    /// Dispatch a set of [KEvents](crate::KEvent)
    pub fn dispatch_events(
        &mut self,
        events: Vec<KEvent>,
        context: &mut KayakRootContext,
        world: &mut World,
    ) {
        // === Dispatch Events === //
        let mut next_events = HashMap::default();
        for mut event in events {
            let mut current_target: Option<WrappedIndex> = Some(WrappedIndex(event.target));
            while let Some(index) = current_target {
                // Create a copy of the event, specific for this node
                // This is to make sure unauthorized changes to the event are not propagated
                // (e.g., changing the event type, removing the target, etc.)
                let mut node_event = KEvent {
                    current_target: index.0,
                    ..event.clone()
                };

                // --- Update State --- //
                Self::insert_event(&mut next_events, &index, node_event.event_type.clone());

                // --- Call Event --- //
                if let Some(mut entity) = world.get_entity_mut(index.0) {
                    if let Some(mut on_event) = entity.take::<OnEvent>() {
                        let mut event_dispatcher_context = EventDispatcherContext {
                            cursor_capture: self.cursor_capture,
                        };
                        (event_dispatcher_context, node_event) = on_event.try_call(
                            event_dispatcher_context,
                            context.widget_state.clone(),
                            context.focus_tree.clone(),
                            index.0,
                            node_event,
                            world,
                        );
                        world.entity_mut(index.0).insert(on_event);
                        event_dispatcher_context.merge(self);

                        // Sometimes events will require systems to be called.
                        // IE OnChange
                        let widget_context = KayakWidgetContext::new(
                            context.tree.clone(),
                            context.context_entities.clone(),
                            context.layout_cache.clone(),
                            context.widget_state.clone(),
                            context.order_tree.clone(),
                            context.index.clone(),
                            None,
                            context.unique_ids.clone(),
                            context.unique_ids_parents.clone(),
                        );
                        node_event.run_on_change(world, widget_context);
                    }
                }

                event.default_prevented |= node_event.default_prevented;

                // --- Propagate Event --- //
                if node_event.should_propagate {
                    if let Ok(node_tree) = context.tree.try_read() {
                        current_target = node_tree.get_parent(index);
                    } else {
                        current_target = None;
                    }
                } else {
                    current_target = None;
                }
            }

            if !event.default_prevented {
                self.execute_default(event, context, world);
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

    /// Generates a stream of [KEvents](crate::KEvent) from a set of [InputEvents](crate::InputEvent)
    fn build_event_stream(
        &mut self,
        input_events: &[InputEvent],
        context: &mut KayakRootContext,
        world: &mut World,
    ) -> Vec<KEvent> {
        let mut event_stream = Vec::<KEvent>::new();
        let mut states: HashMap<EventType, EventState> = HashMap::new();

        if let Ok(node_tree) = context.tree.try_read() {
            let root = if let Some(root) = node_tree.root_node {
                root
            } else {
                return event_stream;
            };

            // === Setup Cursor States === //
            let old_hovered = self.hovered;
            let old_contains_cursor = self.contains_cursor;
            let old_wants_cursor = self.wants_cursor;
            self.hovered = None;
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
                        let pointer_events = Self::resolve_pointer_events(captor, world);

                        match pointer_events {
                            PointerEvents::All | PointerEvents::SelfOnly => {
                                let events = self.process_pointer_events(
                                    input_event,
                                    (captor, 0),
                                    &mut states,
                                    world,
                                    context,
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
                while let Some((current, depth)) = stack.pop() {
                    let mut enter_children = true;

                    if let Some(entity_ref) = world.get_entity(current.0) {
                        if entity_ref.contains::<OnEvent>() {
                            for input_event in input_events {
                                // --- Process Event --- //
                                if matches!(input_event.category(), InputEventCategory::Mouse) {
                                    // A widget's PointerEvents style will determine how it and its children are processed
                                    let pointer_events =
                                        Self::resolve_pointer_events(current, world);

                                    match pointer_events {
                                        PointerEvents::All | PointerEvents::SelfOnly => {
                                            let events = self.process_pointer_events(
                                                input_event,
                                                (current, depth),
                                                &mut states,
                                                world,
                                                context,
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
                        }
                    }
                    // --- Push Children to Stack --- //
                    if enter_children {
                        if let Some(children) = node_tree.children.get(&current) {
                            let mut stack_children = Vec::new();
                            for child in children {
                                let child_z = world
                                    .get_entity(child.0)
                                    .map(|e| e.get::<Node>().map(|node| node.z).unwrap_or(0.0))
                                    .unwrap_or(0.0);
                                stack_children.push((child_z, (*child, depth + 1)));
                            }
                            stack_children.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                            stack.extend(stack_children.iter().map(|c| c.1));
                        }
                    }
                }
            }

            // === Keyboard Events === //
            for input_event in input_events {
                // Keyboard events only care about the currently focused widget so we don't need to run this over every node in the tree
                let events =
                    self.process_keyboard_events(input_event, &mut states, &context.focus_tree);
                event_stream.extend(events);
            }

            // === Additional Events === //
            let mut had_focus_event = false;

            // These events are ones that require a specific target and need the tree to be evaluated before selecting the best match
            for (event_type, state) in states {
                if let Some(node) = state.best_match {
                    event_stream.push(KEvent::new(node.0, event_type.clone()));

                    match event_type {
                        EventType::Focus => {
                            had_focus_event = true;
                            if let Some(current_focus) =
                                context.focus_tree.current().map(WrappedIndex)
                            {
                                if current_focus != node {
                                    event_stream
                                        .push(KEvent::new(current_focus.0, EventType::Blur));
                                }
                            }
                            context.focus_tree.focus(node.0);
                        }
                        EventType::Hover(..) => {
                            self.hovered = Some(node);
                        }
                        _ => {}
                    }
                }
            }

            // --- Blur Event --- //
            if !had_focus_event && input_events.contains(&InputEvent::MouseLeftPress) {
                // A mouse press didn't contain a focus event -> blur
                if let Some(current_focus) = context.focus_tree.current() {
                    event_stream.push(KEvent::new(current_focus, EventType::Blur));
                    context.focus_tree.blur();
                }
            }

            // === Process Cursor States === //
            self.current_mouse_position = self.next_mouse_position;
            self.is_mouse_pressed = self.next_mouse_pressed;

            if self.hovered.is_none() {
                // No change -> revert
                self.hovered = old_hovered;
            }
            if self.contains_cursor.is_none() {
                // No change -> revert
                self.contains_cursor = old_contains_cursor;
            }
            if self.wants_cursor.is_none() {
                // No change -> revert
                self.wants_cursor = old_wants_cursor;
            }
        }

        event_stream
    }

    /// Process the pointer-related events of an input event
    ///
    /// # Arguments
    ///
    /// * `input_event`: The input event
    /// * `tree_node`: The current node to process
    /// * `states`: The map of events to their current state (for selecting best fit)
    /// * `widget_manager`: The widget manager
    /// * `ignore_layout`: Whether to ignore layout (useful for handling captured events)
    ///
    /// returns: Vec<KEvent>
    fn process_pointer_events(
        &mut self,
        input_event: &InputEvent,
        tree_node: TreeNode,
        states: &mut HashMap<EventType, EventState>,
        world: &mut World,
        context: &KayakRootContext,
        ignore_layout: bool,
    ) -> Vec<KEvent> {
        let mut event_stream = Vec::<KEvent>::new();
        let (node, depth) = tree_node;

        // let widget_name = world.entity(node.0).get::<WidgetName>();
        // dbg!(widget_name);

        match input_event {
            InputEvent::MouseMoved(point) => {
                if let Some(layout) = context.get_layout(&node) {
                    let cursor_event = self.get_cursor_event(*point);
                    let was_contained = layout.contains(&self.current_mouse_position);
                    let is_contained = layout.contains(point);
                    if !ignore_layout && was_contained != is_contained {
                        if was_contained {
                            // Mouse out should fire even when
                            event_stream
                                .push(KEvent::new(node.0, EventType::MouseOut(cursor_event)));
                            // Self::update_state(
                            //     states,
                            //     (node, depth),
                            //     &layout,
                            //     EventType::MouseOut(cursor_event),
                            // );
                        } else {
                            // event_stream.push(Event::new(node.0, EventType::MouseIn(cursor_event)));
                            Self::update_state(
                                states,
                                (node, depth),
                                &layout,
                                EventType::MouseIn(cursor_event),
                            );
                        }
                    }
                    if self.contains_cursor.is_none() || !self.contains_cursor.unwrap_or_default() {
                        if let Some(styles) = world.get::<ComputedStyles>(node.0) {
                            // Check if the cursor moved onto a widget that qualifies as one that can contain it
                            if ignore_layout || Self::can_contain_cursor(&styles.0) {
                                self.contains_cursor = Some(is_contained);
                            }
                        }
                    }

                    if self.wants_cursor.is_none() || !self.wants_cursor.unwrap_or_default() {
                        let focusable = world.get::<Focusable>(node.0).is_some();
                        // Check if the cursor moved onto a focusable widget (i.e. one that would want it)
                        if focusable {
                            self.wants_cursor = Some(is_contained);
                        }
                    }

                    // Check for hover eligibility
                    if ignore_layout || is_contained {
                        Self::update_state(
                            states,
                            (node, depth),
                            &layout,
                            EventType::Hover(cursor_event),
                        );
                    }
                }
            }
            InputEvent::MouseLeftPress => {
                if let Some(layout) = context.get_layout(&node) {
                    if ignore_layout || layout.contains(&self.current_mouse_position) {
                        let cursor_event = self.get_cursor_event(self.current_mouse_position);
                        // event_stream.push(Event::new(node.0, EventType::MouseDown(cursor_event)));
                        Self::update_state(
                            states,
                            (node, depth),
                            &layout,
                            EventType::MouseDown(cursor_event),
                        );

                        if world.get::<Focusable>(node.0).is_some() {
                            Self::update_state(states, (node, depth), &layout, EventType::Focus);
                        }

                        if self.has_cursor.is_none() {
                            if let Some(styles) = world.get::<ComputedStyles>(node.0) {
                                // Check if the cursor moved onto a widget that qualifies as one that can contain it
                                if Self::can_contain_cursor(&styles.0) {
                                    self.has_cursor = Some(node);
                                }
                            }
                        }
                    }
                }
            }
            InputEvent::MouseLeftRelease => {
                if let Some(layout) = context.get_layout(&node) {
                    if ignore_layout || layout.contains(&self.current_mouse_position) {
                        let cursor_event = self.get_cursor_event(self.current_mouse_position);
                        // event_stream.push(Event::new(node.0, EventType::MouseUp(cursor_event)));
                        Self::update_state(
                            states,
                            (node, depth),
                            &layout,
                            EventType::MouseUp(cursor_event),
                        );
                        // self.last_clicked.set(node);

                        if Self::contains_event(
                            &self.previous_events,
                            &node,
                            &EventType::MouseDown(cursor_event),
                        ) {
                            Self::update_state(
                                states,
                                (node, depth),
                                &layout,
                                EventType::Click(cursor_event),
                            );
                        }
                    }
                }
            }
            InputEvent::Scroll { dx, dy, is_line } => {
                if let Some(layout) = context.get_layout(&node) {
                    // Check for scroll eligibility
                    if ignore_layout || layout.contains(&self.current_mouse_position) {
                        Self::update_state(
                            states,
                            (node, depth),
                            &layout,
                            EventType::Scroll(ScrollEvent {
                                delta: if *is_line {
                                    ScrollUnit::Line { x: *dx, y: *dy }
                                } else {
                                    ScrollUnit::Pixel { x: *dx, y: *dy }
                                },
                            }),
                        );
                    }
                }
            }
            _ => {}
        }

        event_stream
    }

    fn resolve_pointer_events(index: WrappedIndex, world: &mut World) -> PointerEvents {
        let mut pointer_events = PointerEvents::default();
        if let Some(styles) = world.get::<ComputedStyles>(index.0) {
            pointer_events = styles.0.pointer_events.resolve();
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
        focus_tree: &FocusTree,
    ) -> Vec<KEvent> {
        let mut event_stream = Vec::new();
        if let Some(current_focus) = focus_tree.current() {
            match input_event {
                InputEvent::CharEvent { c } => event_stream.push(KEvent::new(
                    current_focus,
                    EventType::CharInput { c: c.clone() },
                )),
                InputEvent::Keyboard { key, is_pressed } => {
                    // === Modifers === //
                    match key {
                        KeyCode::ControlLeft | KeyCode::ControlRight => {
                            self.keyboard_modifiers.is_ctrl_pressed = *is_pressed
                        }
                        KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                            self.keyboard_modifiers.is_shift_pressed = *is_pressed
                        }
                        KeyCode::AltLeft | KeyCode::AltRight => {
                            self.keyboard_modifiers.is_alt_pressed = *is_pressed
                        }
                        KeyCode::SuperLeft | KeyCode::SuperRight => {
                            self.keyboard_modifiers.is_meta_pressed = *is_pressed
                        }
                        _ => {}
                    }

                    // === Event === //
                    if *is_pressed {
                        event_stream.push(KEvent::new(
                            current_focus,
                            EventType::KeyDown(KeyboardEvent::new(*key, self.keyboard_modifiers)),
                        ))
                    } else {
                        event_stream.push(KEvent::new(
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
        let state = states.entry(event_type).or_default();

        let (node, depth) = tree_node;
        // Node is at or above best depth and is at or above best z-level

        let z_index = layout.z_index.unwrap_or(0.0);

        let mut should_update = depth >= state.best_depth && z_index >= state.best_z_index;
        // OR node is above best z-level
        should_update |= z_index >= state.best_z_index;

        if should_update {
            // dbg!(node.0, layout.z_index);
            state.best_match = Some(node);
            state.best_z_index = z_index;
            state.best_depth = depth;
        }
    }

    /// Checks if the given event map contains a specific event for the given widget
    fn contains_event(events: &EventMap, widget_id: &WrappedIndex, event_type: &EventType) -> bool {
        if let Some(entry) = events.get(widget_id) {
            entry.contains(event_type)
        } else {
            false
        }
    }

    /// Insert an event for a widget in the given event map
    fn insert_event(
        events: &mut EventMap,
        widget_id: &WrappedIndex,
        event_type: EventType,
    ) -> bool {
        let entry = events.entry(*widget_id).or_default();
        entry.insert(event_type)
    }

    /// Checks if the given widget is eligible to "contain" the cursor (i.e. the cursor is considered contained when hovering over it)
    ///
    /// Currently a valid widget is defined as one where:
    /// * RenderCommands is neither `Empty` nor `Layout` nor `Clip`
    fn can_contain_cursor(widget_styles: &KStyle) -> bool {
        let cmds = widget_styles.render_command.resolve();
        !matches!(
            cmds,
            RenderCommand::Empty | RenderCommand::Layout | RenderCommand::Clip
        )
    }

    /// Executes default actions for events
    fn execute_default(
        &mut self,
        event: KEvent,
        context: &mut KayakRootContext,
        world: &mut World,
    ) {
        if let EventType::KeyDown(evt) = event.event_type {
            if let KeyCode::Tab = evt.key() {
                let (index, current_focus) = {
                    let current_focus = context.focus_tree.current();

                    let index = if evt.is_shift_pressed() {
                        context.focus_tree.prev()
                    } else {
                        context.focus_tree.next()
                    };
                    (index, current_focus)
                };

                if let Some(index) = index {
                    let mut events = vec![KEvent::new(index, EventType::Focus)];
                    if let Some(current_focus) = current_focus {
                        if current_focus != index {
                            events.push(KEvent::new(current_focus, EventType::Blur));
                        }
                    }
                    context.focus_tree.focus(index);
                    self.dispatch_events(events, context, world);
                }
            }
        }
    }

    /// Merge this `EventDispatcher` with another, taking only the internally mutated data.
    ///
    /// This is meant to solve the issue in `Context`, where [`EventDispatcher::process_events`] and
    /// similar methods require mutable access to `Context`, forcing `EventDispatcher` to be cloned
    /// before running the method. However, some data mutated through `Context` may be lost when
    /// re-claiming the `EventDispatcher`. This method ensures that data mutated in such a way will not be
    /// overwritten during the merge.
    ///
    /// # Arguments
    ///
    /// * `from`: The other `EventDispatcher` to merge from
    ///
    /// returns: ()
    pub fn merge(&mut self, from: EventDispatcher) {
        // Merge only what could be changed internally. External changes (i.e. from Context)
        // should not be touched
        // self.last_clicked = from.last_clicked;
        self.is_mouse_pressed = from.is_mouse_pressed;
        self.next_mouse_pressed = from.next_mouse_pressed;
        self.current_mouse_position = from.current_mouse_position;
        self.next_mouse_position = from.next_mouse_position;
        self.previous_events = from.previous_events;
        self.keyboard_modifiers = from.keyboard_modifiers;
        self.contains_cursor = from.contains_cursor;
        self.wants_cursor = from.wants_cursor;
        self.has_cursor = from.has_cursor;
        self.hovered = from.hovered;

        // Do not include:
        // self.cursor_capture = from.cursor_capture;
    }
}

#[derive(Resource, Default)]
pub struct EventDispatcherContext {
    cursor_capture: Option<WrappedIndex>,
}

impl EventDispatcherContext {
    /// Captures all cursor events and instead makes the given index the target
    pub fn capture_cursor(&mut self, index: Entity) -> Option<WrappedIndex> {
        let old = self.cursor_capture;
        self.cursor_capture = Some(WrappedIndex(index));
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
    pub fn release_cursor(&mut self, index: Entity) -> bool {
        if self.cursor_capture == Some(WrappedIndex(index)) {
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
    pub fn force_release_cursor(&mut self) -> Option<WrappedIndex> {
        let old = self.cursor_capture;
        self.cursor_capture = None;
        old
    }

    pub(crate) fn merge(self, event_dispatcher: &mut EventDispatcher) {
        event_dispatcher.cursor_capture = self.cursor_capture;
    }
}
