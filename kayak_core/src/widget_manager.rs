use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use crate::layout_cache::Rect;
use crate::{focus_tree::FocusTracker, focus_tree::FocusTree, layout_cache::LayoutCache, node::{Node, NodeBuilder}, render_command::RenderCommand, render_primitive::RenderPrimitive, styles::Style, tree::Tree, Arena, Index, Widget, WidgetProps, BoxedWidget};
// use as_any::Downcast;

#[derive(Debug)]
pub struct WidgetManager {
    pub(crate) current_widgets: Arena<Option<BoxedWidget>>,
    pub(crate) dirty_render_nodes: HashSet<Index>,
    pub(crate) dirty_nodes: Arc<Mutex<HashSet<Index>>>,
    pub(crate) nodes: Arena<Option<Node>>,
    /// A tree containing all widgets in the hierarchy.
    pub tree: Tree,
    /// A tree containing only the widgets with layouts in the hierarchy.
    pub node_tree: Tree,
    /// A tree containing all actively focusable widgets.
    pub focus_tree: FocusTree,
    pub layout_cache: LayoutCache,
    focus_tracker: FocusTracker,
    current_z: f32,
}

impl WidgetManager {
    pub fn new() -> Self {
        Self {
            current_widgets: Arena::new(),
            dirty_render_nodes: HashSet::new(),
            dirty_nodes: Arc::new(Mutex::new(HashSet::new())),
            nodes: Arena::new(),
            tree: Tree::default(),
            node_tree: Tree::default(),
            layout_cache: LayoutCache::default(),
            focus_tree: FocusTree::default(),
            focus_tracker: FocusTracker::default(),
            current_z: 0.0,
        }
    }

    /// Re-renders from the root.
    /// If force is true sets ALL nodes to re-render.
    /// Can be slow.
    pub fn dirty(&mut self, force: bool) {
        // Force tree to re-render from root.
        if let Ok(mut dirty_nodes) = self.dirty_nodes.lock() {
            dirty_nodes.insert(self.tree.root_node.unwrap());

            if force {
                for (node_index, _) in self.current_widgets.iter() {
                    dirty_nodes.insert(node_index);
                    self.dirty_render_nodes.insert(node_index);
                }
            }
        }
    }

    pub fn create_widget<T: Widget + 'static>(
        &mut self,
        index: usize,
        mut widget: T,
        parent: Option<Index>,
    ) -> (bool, Index) {
        let widget_id = if let Some(parent) = parent.clone() {
            if let Some(parent_children) = self.tree.children.get_mut(&parent) {
                parent_children.get(index).cloned()
            } else {
                None
            }
        } else {
            None
        };

        // Pull child and update.
        if let Some(widget_id) = widget_id {
            widget.set_id(widget_id);
            // Remove from the dirty nodes lists.
            // if let Some(index) = self.dirty_nodes.iter().position(|id| *widget_id == *id) {
            //     self.dirty_nodes.remove(index);
            // }

            // Mark this widget as focusable if it's designated focusable or if it's the root node
            if self.tree.is_empty() {
                self.set_focusable(Some(true), widget_id, true);
            } else {
                self.set_focusable(widget.get_props().get_focusable(), widget_id, true);
            }

            // TODO: Figure a good way of diffing props passed to children of a widget
            // that wont naturally-rerender it's children because of a lack of changes
            // to it's own props.
            // if &widget
            //     != self.current_widgets[*widget_id]
            //         .as_ref()
            //         .unwrap()
            //         .downcast_ref::<T>()
            //         .unwrap()
            // {
            let boxed_widget: BoxedWidget = Box::new(widget);
            *self.current_widgets[widget_id].as_mut().unwrap() = boxed_widget;
            // Tell renderer that the nodes changed.
            self.dirty_render_nodes.insert(widget_id);
            return (true, widget_id);
            // } else {
            //     return (false, *widget_id);
            // }
        }

        // Mark this widget as focusable if it's designated focusable or if it's the root node
        let focusable = if self.tree.is_empty() {
            Some(true)
        } else {
            widget.get_props().get_focusable()
        };

        // Create Flow
        // We should only have one widget that doesn't have a parent.
        // The root widget.
        let widget_id = self.current_widgets.insert(Some(Box::new(widget)));
        self.nodes.insert(None);
        self.current_widgets[widget_id]
            .as_mut()
            .unwrap()
            .set_id(widget_id);

        // Tell renderer that the nodes changed.
        self.dirty_render_nodes.insert(widget_id);

        // Remove from the dirty nodes lists.
        // if let Some(index) = self.dirty_nodes.iter().position(|id| widget_id == *id) {
        //     self.dirty_nodes.remove(index);
        // }

        self.tree.add(widget_id, parent);
        self.layout_cache.add(widget_id);
        self.set_focusable(focusable, widget_id, true);

        (true, widget_id)
    }

    pub fn take(&mut self, id: Index) -> BoxedWidget {
        self.current_widgets[id].take().unwrap()
    }

    pub fn repossess(&mut self, widget: BoxedWidget) {
        let widget_id = widget.get_id();
        self.current_widgets[widget_id] = Some(widget);
    }

    pub fn get_layout(&self, id: &Index) -> Option<&Rect> {
        self.layout_cache.rect.get(id)
    }

    pub fn get_name(&self, id: &Index) -> Option<String> {
        if let Some(widget) = &self.current_widgets[*id] {
            return Some(widget.get_name().to_string());
        }

        None
    }

    pub fn render(&mut self) {
        let default_styles = Style {
            background_color: crate::styles::StyleProp::Default,
            border_radius: crate::styles::StyleProp::Default,
            bottom: crate::styles::StyleProp::Default,
            color: crate::styles::StyleProp::Default,
            height: crate::styles::StyleProp::Default,
            layout_type: crate::styles::StyleProp::Default,
            left: crate::styles::StyleProp::Default,
            padding_bottom: crate::styles::StyleProp::Default,
            padding_left: crate::styles::StyleProp::Default,
            padding_right: crate::styles::StyleProp::Default,
            padding_top: crate::styles::StyleProp::Default,
            position_type: crate::styles::StyleProp::Default,
            render_command: crate::styles::StyleProp::Default,
            right: crate::styles::StyleProp::Default,
            top: crate::styles::StyleProp::Default,
            width: crate::styles::StyleProp::Default,
            ..Style::default()
        };
        for dirty_node_index in self.dirty_render_nodes.drain() {
            let dirty_widget = self.current_widgets[dirty_node_index].as_ref().unwrap();
            let parent_styles =
                if let Some(parent_widget_id) = self.tree.parents.get(&dirty_node_index) {
                    if let Some(parent) = self.current_widgets[*parent_widget_id].as_ref() {
                        if let Some(styles) = parent.get_props().get_styles() {
                            styles
                        } else {
                            default_styles.clone()
                        }
                    } else {
                        default_styles.clone()
                    }
                } else {
                    default_styles.clone()
                };

            // Get parent Z
            let parent_z = if let Some(parent_widget_id) = self.tree.parents.get(&dirty_node_index)
            {
                if let Some(parent) = &self.nodes[*parent_widget_id] {
                    parent.z
                } else {
                    -1.0
                }
            } else {
                -1.0
            };

            let current_z = {
                if parent_z > -1.0 {
                    parent_z + 1.0
                } else {
                    let z = self.current_z;
                    self.current_z += 1.0;
                    z
                }
            };

            let mut styles = dirty_widget.get_props().get_styles();
            if styles.is_some() {
                styles.as_mut().unwrap().merge(&parent_styles);
            }
            let children = self
                .tree
                .children
                .get(&dirty_node_index)
                .cloned()
                .unwrap_or(vec![]);
            let styles = styles.unwrap_or(default_styles.clone());

            let mut node = NodeBuilder::empty()
                .with_id(dirty_node_index)
                .with_styles(styles)
                .with_children(children)
                .build();
            node.z = current_z;

            self.nodes[dirty_node_index] = Some(node);
        }

        self.node_tree = self.build_nodes_tree();
    }

    pub fn calculate_layout(&mut self) {
        morphorm::layout(&mut self.layout_cache, &self.node_tree, &self.nodes);
    }

    fn recurse_node_tree_to_build_primitives(
        node_tree: &Tree,
        layout_cache: &LayoutCache,
        nodes: &Arena<Option<Node>>,
        current_node: Index,
        mut main_z_index: f32,
        mut prev_clip: RenderPrimitive,
    ) -> Vec<RenderPrimitive> {
        let mut render_primitives = Vec::new();

        if let Some(node) = nodes.get(current_node).unwrap() {
            if let Some(layout) = layout_cache.rect.get(&current_node) {
                let mut render_primitive: RenderPrimitive = (&node.styles).into();
                let mut layout = *layout;
                let new_z_index = if matches!(render_primitive, RenderPrimitive::Clip { .. }) {
                    main_z_index - 0.1
                } else {
                    main_z_index
                };
                layout.z_index = new_z_index;
                render_primitive.set_layout(layout);
                render_primitives.push(render_primitive.clone());

                let new_prev_clip = if matches!(render_primitive, RenderPrimitive::Clip { .. }) {
                    render_primitive.clone()
                } else {
                    prev_clip
                };

                prev_clip = new_prev_clip.clone();

                if node_tree.children.contains_key(&current_node) {
                    for child in node_tree.children.get(&current_node).unwrap() {
                        main_z_index += 1.0;
                        render_primitives.extend(Self::recurse_node_tree_to_build_primitives(
                            node_tree,
                            layout_cache,
                            nodes,
                            *child,
                            main_z_index,
                            new_prev_clip.clone(),
                        ));

                        main_z_index = layout.z_index;
                        // Between each child node we need to reset the clip.
                        if matches!(prev_clip, RenderPrimitive::Clip { .. }) {
                            // main_z_index = new_z_index;
                            match &mut prev_clip {
                                RenderPrimitive::Clip { layout } => {
                                    layout.z_index = main_z_index + 0.1;
                                }
                                _ => {}
                            };
                            render_primitives.push(prev_clip.clone());
                        }
                    }
                }
            }
        }

        render_primitives
    }

    pub fn build_render_primitives(&self) -> Vec<RenderPrimitive> {
        Self::recurse_node_tree_to_build_primitives(
            &self.node_tree,
            &self.layout_cache,
            &self.nodes,
            self.node_tree.root_node.unwrap(),
            0.0,
            RenderPrimitive::Empty,
        )
    }

    fn build_nodes_tree(&mut self) -> Tree {
        let mut tree = Tree::default();
        let (root_node_id, _) = self.current_widgets.iter().next().unwrap();
        tree.root_node = Some(root_node_id);
        tree.children.insert(
            tree.root_node.unwrap(),
            self.get_valid_node_children(tree.root_node.unwrap()),
        );

        let old_focus = self.focus_tree.current();
        self.focus_tree.clear();
        self.focus_tree.add(root_node_id, &self.tree);

        for (widget_id, widget) in self.current_widgets.iter().skip(1) {
            let widget_styles = widget.as_ref().unwrap().get_props().get_styles();
            if let Some(widget_styles) = widget_styles {
                // Only add widgets who have renderable nodes.
                if widget_styles.render_command.resolve() != RenderCommand::Empty {
                    let valid_children = self.get_valid_node_children(widget_id);
                    tree.children.insert(widget_id, valid_children);
                    let valid_parent = self.get_valid_parent(widget_id);
                    if let Some(valid_parent) = valid_parent {
                        tree.parents.insert(widget_id, valid_parent);
                    }
                }
            }

            let focusable = self.get_focusable(widget_id).unwrap_or_default();
            if focusable {
                self.focus_tree.add(widget_id, &self.tree);
            }
        }

        if let Some(old_focus) = old_focus {
            if self.focus_tree.contains(old_focus) {
                self.focus_tree.focus(old_focus);
            }
        }

        tree
    }

    fn get_valid_node_children(&self, node_id: Index) -> Vec<Index> {
        let mut children = Vec::new();
        if let Some(node_children) = self.tree.children.get(&node_id) {
            for child_id in node_children {
                if let Some(child_widget) = &self.current_widgets[*child_id] {
                    if let Some(child_styles) = child_widget.get_props().get_styles() {
                        if child_styles.render_command.resolve() != RenderCommand::Empty {
                            children.push(*child_id);
                        } else {
                            children.extend(self.get_valid_node_children(*child_id));
                        }
                    } else {
                        children.extend(self.get_valid_node_children(*child_id));
                    }
                }
            }
        }

        children
    }

    pub fn get_valid_parent(&self, node_id: Index) -> Option<Index> {
        if let Some(parent_id) = self.tree.parents.get(&node_id) {
            if let Some(parent_widget) = &self.nodes[*parent_id] {
                if parent_widget.styles.render_command.resolve() != RenderCommand::Empty {
                    return Some(*parent_id);
                }
            }
            return self.get_valid_parent(*parent_id);
        }
        // assert!(node_id.into_raw_parts().0 == 0);
        None
    }

    pub fn get_node(&self, id: &Index) -> Option<Node> {
        self.nodes[*id].clone()
    }

    pub fn get_focusable(&self, index: Index) -> Option<bool> {
        self.focus_tracker.get_focusability(index)
    }

    pub fn set_focusable(&mut self, focusable: Option<bool>, index: Index, is_parent: bool) {
        self.focus_tracker
            .set_focusability(index, focusable, is_parent);
    }
}
