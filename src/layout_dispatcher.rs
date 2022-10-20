use bevy::{
    prelude::{Entity, With, World},
    utils::HashSet,
};
use indexmap::IndexMap;
use morphorm::GeometryChanged;

use crate::{
    layout::{LayoutCache, LayoutEvent},
    node::WrappedIndex,
    on_layout::OnLayout,
    prelude::Context,
};

pub(crate) struct LayoutEventDispatcher;

impl LayoutEventDispatcher {
    pub fn dispatch(context: &mut Context, world: &mut World) {
        let on_event_entities = {
            let mut query = world.query_filtered::<Entity, With<OnLayout>>();
            query
                .iter(world)
                .map(|entity| entity)
                .collect::<HashSet<_>>()
        };

        if let Ok(layout_cache) = context.layout_cache.try_read() {
            let changed = layout_cache.iter_changed();
            let changed = changed
                .filter_map(|(index, flags)| {
                    if on_event_entities.contains(&index.0) {
                        Some((*index, *flags))
                    } else {
                        None
                    }
                })
                .collect::<IndexMap<WrappedIndex, GeometryChanged>>();

            // Use IndexSet to prevent duplicates and maintain speed
            let mut parents: IndexMap<WrappedIndex, GeometryChanged> = IndexMap::default();

            if let Ok(tree) = context.tree.try_read() {
                for (node_index, flags) in &changed {
                    // Add parent to set
                    if let Some(parent_index) = tree.get_parent(*node_index) {
                        if !changed.contains_key(&parent_index) {
                            parents.insert(parent_index, GeometryChanged::default());
                        }
                    }

                    // Process and dispatch
                    Self::process(world, &layout_cache, *node_index, *flags);
                }
            }

            // Finally, process all parents
            for (parent_index, flags) in parents {
                // Process and dispatch
                Self::process(world, &layout_cache, parent_index, flags);
            }
        }
    }

    fn process(
        world: &mut World,
        layout_cache: &LayoutCache,
        index: WrappedIndex,
        flags: GeometryChanged,
    ) {
        // We should be able to just get layout from WidgetManager here
        // since the layouts will be calculated by this point
        if let Some(mut entity) = world.get_entity_mut(index.0) {
            if let Some(mut on_layout) = entity.remove::<OnLayout>() {
                if let Some(rect) = layout_cache.rect.get(&index) {
                    // dbg!(format!("Processing event for: {:?}", entity.id()));
                    let layout_event = LayoutEvent::new(*rect, flags, index.0);
                    on_layout.try_call(index.0, layout_event, world);
                    world.entity_mut(index.0).insert(on_layout);
                }
            }
        }
    }
}
