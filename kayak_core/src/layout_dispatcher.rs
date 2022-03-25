use indexmap::IndexSet;
use morphorm::{Cache};

use crate::{Index, KayakContext, KayakContextRef, LayoutEvent};

pub(crate) struct LayoutEventDispatcher;

impl LayoutEventDispatcher {
    pub fn dispatch(context: &mut KayakContext) {
        let dirty_nodes: Vec<Index> = context
            .widget_manager
            .dirty_render_nodes
            .drain(..)
            .collect();

        // Use IndexSet to prevent duplicates and maintain speed
        let mut parents: IndexSet<Index> = IndexSet::default();

        for node_index in dirty_nodes {
            // If layout is not changed -> skip
            if context
                .widget_manager
                .layout_cache
                .geometry_changed(node_index)
                .is_empty()
            {
                continue;
            }

            // Add parent to set
            if let Some(parent_index) = context.widget_manager.tree.get_parent(node_index) {
                parents.insert(parent_index);
            }

            // Process and dispatch
            Self::process(node_index, context);
        }

        // Finally, process all parents
        for parent_index in parents {
            // Process and dispatch
            Self::process(parent_index, context);
        }
    }

    fn process(index: Index, context: &mut KayakContext) {
        // We should be able to just get layout from WidgetManager here
        // since the layouts will be calculated by this point
        let widget = context.widget_manager.take(index);
        if let Some(on_layout) = widget.get_props().get_on_layout() {
            if let Some(rect) = context.widget_manager.layout_cache.rect.get(&index) {
                let layout_event = LayoutEvent::new(
                    *rect,
                    context.widget_manager.layout_cache.geometry_changed(index),
                    index,
                );
                let mut context_ref = KayakContextRef::new(context, Some(index));

                on_layout.try_call(&mut context_ref, &layout_event);
            }
        }

        context.widget_manager.repossess(widget);
    }
}
