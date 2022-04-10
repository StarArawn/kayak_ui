use indexmap::IndexMap;
use morphorm::GeometryChanged;

use crate::{Index, KayakContext, KayakContextRef, LayoutEvent};

pub(crate) struct LayoutEventDispatcher;

impl LayoutEventDispatcher {
    pub fn dispatch(context: &mut KayakContext) {
        let changed = context
            .widget_manager
            .layout_cache
            .iter_changed()
            .map(|(index, flags)| (*index, *flags))
            .collect::<IndexMap<Index, GeometryChanged>>();

        // Use IndexSet to prevent duplicates and maintain speed
        let mut parents: IndexMap<Index, GeometryChanged> = IndexMap::default();

        for (node_index, flags) in &changed {
            // Add parent to set
            if let Some(parent_index) = context.widget_manager.tree.get_parent(*node_index) {
                if !changed.contains_key(&parent_index) {
                    parents.insert(parent_index, GeometryChanged::default());
                }
            }

            // Process and dispatch
            Self::process(*node_index, *flags, context);
        }

        // Finally, process all parents
        for (parent_index, flags) in parents {
            // Process and dispatch
            Self::process(parent_index, flags, context);
        }
    }

    fn process(index: Index, flags: GeometryChanged, context: &mut KayakContext) {
        // We should be able to just get layout from WidgetManager here
        // since the layouts will be calculated by this point
        let widget = context.widget_manager.take(index);
        if let Some(on_layout) = widget.get_props().get_on_layout() {
            if let Some(rect) = context.widget_manager.layout_cache.rect.get(&index) {
                let layout_event = LayoutEvent::new(*rect, flags, index);
                let mut context_ref = KayakContextRef::new(context, Some(index));

                on_layout.try_call(&mut context_ref, &layout_event);
            }
        }

        context.widget_manager.repossess(widget);
    }
}
