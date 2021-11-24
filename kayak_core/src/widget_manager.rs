use crate::{
    layout_cache::LayoutCache,
    node::{Node, NodeBuilder, NodeIndex},
    render_primitive::RenderPrimitive,
    styles::Style,
    tree::Tree,
    Arena, Index, Widget,
};
use as_any::Downcast;

#[derive(Debug)]
pub struct WidgetManager {
    current_widgets: Arena<Option<Box<dyn Widget>>>,
    pub(crate) dirty_render_nodes: Vec<Index>,
    pub(crate) dirty_nodes: Vec<Index>,
    pub(crate) nodes: Arena<Option<Node>>,
    pub tree: Tree,
    pub layout_cache: LayoutCache,
    current_z: f32,
}

impl WidgetManager {
    pub fn new() -> Self {
        Self {
            current_widgets: Arena::new(),
            dirty_render_nodes: Vec::new(),
            dirty_nodes: Vec::new(),
            nodes: Arena::new(),
            tree: Tree::default(),
            layout_cache: LayoutCache::default(),
            current_z: 0.0,
        }
    }

    /// Re-renders from the root.
    /// If force is true sets ALL nodes to re-render.
    /// Can be slow.
    pub fn dirty(&mut self, force: bool) {
        // Force tree to re-render from root.
        self.dirty_nodes.push(self.tree.root_node);

        if force {
            for (node_index, _) in self.current_widgets.iter() {
                self.dirty_nodes.push(node_index);
                self.dirty_render_nodes.push(node_index);
            }
        }
    }

    pub fn create_widget<T: Widget + PartialEq + 'static>(
        &mut self,
        index: usize,
        mut widget: T,
        parent: Option<Index>,
    ) -> (bool, Index) {
        if let Some(parent) = parent.clone() {
            if let Some(parent_children) = self.tree.children.get_mut(&parent) {
                // Pull child and update.
                if let Some(widget_id) = parent_children.get(index) {
                    widget.set_id(*widget_id);
                    // Remove from the dirty nodes lists.
                    if let Some(index) = self.dirty_nodes.iter().position(|id| *widget_id == *id) {
                        self.dirty_nodes.remove(index);
                    }

                    if &widget
                        != self.current_widgets[*widget_id]
                            .as_ref()
                            .unwrap()
                            .downcast_ref::<T>()
                            .unwrap()
                    {
                        let boxed_widget: Box<dyn Widget> = Box::new(widget);
                        *self.current_widgets[*widget_id].as_mut().unwrap() = boxed_widget;
                        dbg!("Widget changed!");
                        // Tell renderer that the nodes changed.
                        self.dirty_render_nodes.push(*widget_id);
                        return (true, *widget_id);
                    } else {
                        dbg!("No widget changes!");
                        return (false, *widget_id);
                    }
                }
            }
        }

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
        self.dirty_render_nodes.push(widget_id);

        // Remove from the dirty nodes lists.
        if let Some(index) = self.dirty_nodes.iter().position(|id| widget_id == *id) {
            self.dirty_nodes.remove(index);
        }

        self.tree.add(0, widget_id, parent);
        self.layout_cache.add(NodeIndex(widget_id));

        (true, widget_id)
    }

    pub fn take(&mut self, id: Index) -> Box<dyn Widget> {
        self.current_widgets[id].take().unwrap()
    }

    pub fn repossess(&mut self, widget: Box<dyn Widget>) {
        let widget_id = widget.get_id();
        self.current_widgets[widget_id] = Some(widget);
    }

    pub fn render(&mut self) {
        let default_styles = Style {
            background_color: crate::styles::StyleProp::Default,
            bottom: crate::styles::StyleProp::Default,
            color: crate::styles::StyleProp::Default,
            height: crate::styles::StyleProp::Default,
            layout_type: crate::styles::StyleProp::Default,
            left: crate::styles::StyleProp::Default,
            position_type: crate::styles::StyleProp::Default,
            render_command: crate::styles::StyleProp::Default,
            right: crate::styles::StyleProp::Default,
            top: crate::styles::StyleProp::Default,
            width: crate::styles::StyleProp::Default,
        };
        for dirty_node_index in self.dirty_render_nodes.drain(..) {
            let dirty_widget = self.current_widgets[dirty_node_index].as_ref().unwrap();
            let parent_styles =
                if let Some(parent_widget_id) = self.tree.parents.get(&dirty_node_index) {
                    if let Some(parent) = self.current_widgets[*parent_widget_id].as_ref() {
                        if let Some(styles) = parent.get_styles() {
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
            dbg!(current_z);

            let mut styles = dirty_widget.get_styles();
            if styles.is_some() {
                styles.as_mut().unwrap().merge(&parent_styles);
            }
            let children = self
                .tree
                .children
                .get(&dirty_node_index)
                .cloned()
                .unwrap_or(vec![]);
            let mut node = NodeBuilder::empty()
                .with_id(dirty_node_index)
                .with_styles(styles.unwrap_or(default_styles.clone()))
                .with_children(children)
                .build();
            node.z = current_z;

            self.nodes[dirty_node_index] = Some(node);
        }
    }

    pub fn calculate_layout(&mut self) {
        morphorm::layout(&mut self.layout_cache, &self.tree, &self.nodes);
    }

    pub fn build_render_primitives(&self) -> Vec<RenderPrimitive> {
        let mut render_primitives = Vec::new();

        for (index, node) in self.nodes.iter() {
            if let Some(layout) = self.layout_cache.rect.get(&NodeIndex(index)) {
                if let Some(node) = node {
                    let mut render_primitive: RenderPrimitive = (&node.styles).into();
                    let mut layout = *layout;
                    layout.z_index = node.z;
                    render_primitive.set_layout(layout);
                    render_primitives.push(render_primitive);
                }
            }
        }

        render_primitives
    }
}
