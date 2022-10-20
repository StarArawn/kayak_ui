use bevy::{
    prelude::{Assets, Commands, Entity, Query, Res, ResMut, With},
    utils::HashMap,
};
use kayak_font::KayakFont;

use crate::{
    layout::{DataCache, Rect},
    node::{DirtyNode, Node, NodeBuilder, WrappedIndex},
    prelude::{Context, KStyle},
    render::font::FontMapping,
    render_primitive::RenderPrimitive,
    styles::{StyleProp, Units},
};

pub fn calculate_nodes(
    mut commands: Commands,
    mut context: ResMut<Context>,
    fonts: Res<Assets<KayakFont>>,
    font_mapping: Res<FontMapping>,
    query: Query<Entity, With<DirtyNode>>,
    all_styles_query: Query<&KStyle>,
    node_query: Query<(Entity, &Node)>,
    nodes_no_entity_query: Query<&'static Node>,
) {
    let mut new_nodes = HashMap::<Entity, (Node, bool)>::default();
    // This is the maximum recursion depth for this method.
    // Recursion involves recalculating layout which should be done sparingly.
    // const MAX_RECURSION_DEPTH: usize = 2;

    context.current_z = 0.0;

    let initial_styles = KStyle::initial();
    let default_styles = KStyle::new_default();

    // Jump out early.
    // if query.is_empty() {
    //     return;
    // }
    if let Ok(tree) = context.tree.clone().read() {
        for dirty_entity in query.iter() {
            let dirty_entity = WrappedIndex(dirty_entity);
            let styles = all_styles_query
                .get(dirty_entity.0)
                .unwrap_or(&default_styles);
            // Get the parent styles. Will be one of the following:
            // 1. Already-resolved node styles (best)
            // 2. Unresolved widget prop styles
            // 3. Unresolved default styles
            let parent_styles = if let Some(parent_widget_id) = tree.parents.get(&dirty_entity) {
                if let Ok((_, parent_node)) = node_query.get(parent_widget_id.0) {
                    parent_node.resolved_styles.clone()
                } else if let Some(parent_node) = new_nodes.get(&parent_widget_id.0) {
                    parent_node.0.resolved_styles.clone()
                } else if let Ok(parent_styles) = all_styles_query.get(parent_widget_id.0) {
                    parent_styles.clone()
                } else {
                    default_styles.clone()
                }
            } else {
                default_styles.clone()
            };

            let parent_z = if let Some(parent_widget_id) = tree.parents.get(&dirty_entity) {
                if let Ok((_, parent_node)) = node_query.get(parent_widget_id.0) {
                    parent_node.z
                } else if let Some(parent_node) = new_nodes.get(&parent_widget_id.0) {
                    parent_node.0.z
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
                    let z = context.current_z;
                    context.current_z += 1.0;
                    z
                }
            };

            let raw_styles = styles.clone();
            let mut styles = raw_styles.clone();
            // Fill in all `initial` values for any unset property
            styles.apply(&initial_styles);
            // Fill in all `inherited` values for any `inherit` property
            styles.inherit(&parent_styles);

            let (primitive, needs_layout) = create_primitive(
                &mut commands,
                &context,
                &fonts,
                &font_mapping,
                // &node_query,
                dirty_entity,
                &mut styles,
            );

            let children = tree.children.get(&dirty_entity).cloned().unwrap_or(vec![]);

            let width = styles.width.resolve().value_or(0.0, 0.0);
            let height = styles.height.resolve().value_or(0.0, 0.0);

            let mut node = NodeBuilder::empty()
                .with_id(dirty_entity)
                .with_styles(styles, Some(raw_styles))
                .with_children(children)
                .with_primitive(primitive)
                .build();

            if dirty_entity == tree.root_node.unwrap() {
                if let Ok(mut cache) = context.layout_cache.try_write() {
                    cache.rect.insert(
                        dirty_entity,
                        Rect {
                            posx: 0.0,
                            posy: 0.0,
                            width,
                            height,
                            z_index: 0.0,
                        },
                    );
                }
            }
            node.z = current_z;
            new_nodes.insert(dirty_entity.0, (node, needs_layout));
        }

        // let has_new_nodes = new_nodes.len() > 0;

        for (entity, (node, needs_layout)) in new_nodes.drain() {
            commands.entity(entity).insert(node);
            if !needs_layout {
                commands.entity(entity).remove::<DirtyNode>();
            }
        }

        // if has_new_nodes {
        // build_nodes_tree(&mut context, &tree, &node_query);
        // }

        // dbg!("STARTING MORPHORM CALC!");
        // dbg!("node_tree");
        // context.node_tree.dump();
        // if let Ok(tree) = context.tree.try_read() {
        // dbg!("tree");
        // dbg!(&tree);
        // tree.dump();
        // }
        {
            let context = context.as_mut();
            if let Ok(tree) = context.tree.try_read() {
                let node_tree = &*tree;
                if let Ok(mut cache) = context.layout_cache.try_write() {
                    let mut data_cache = DataCache {
                        cache: &mut cache,
                        query: &nodes_no_entity_query,
                    };

                    // dbg!(&node_tree);

                    morphorm::layout(&mut data_cache, node_tree, &nodes_no_entity_query);
                }
            }
        }
        // dbg!("FINISHED MORPHORM CALC!");
    }
}

fn create_primitive(
    commands: &mut Commands,
    context: &Context,
    fonts: &Assets<KayakFont>,
    font_mapping: &FontMapping,
    // query: &Query<(Entity, &Node)>,
    id: WrappedIndex,
    styles: &mut KStyle,
) -> (RenderPrimitive, bool) {
    let mut render_primitive = RenderPrimitive::from(&styles.clone());
    let mut needs_layout = false;

    match &mut render_primitive {
        RenderPrimitive::Text {
            content,
            font,
            properties,
            text_layout,
            ..
        } => {
            // --- Bind to Font Asset --- //
            let font_handle = font_mapping.get_handle(font.clone()).unwrap();
            if let Some(font) = fonts.get(&font_handle) {
                // self.bind(id, &asset);
                if let Ok(node_tree) = context.tree.try_read() {
                    if let Some(parent_id) = node_tree.get_parent(id) {
                        if let Some(parent_layout) = context.get_layout(&parent_id) {
                            properties.max_size = (parent_layout.width, parent_layout.height);

                            // --- Calculate Text Layout --- //
                            *text_layout = font.measure(&content, *properties);
                            let measurement = text_layout.size();

                            // --- Apply Layout --- //
                            if matches!(styles.width, StyleProp::Default) {
                                styles.width = StyleProp::Value(Units::Pixels(measurement.0));
                            }
                            if matches!(styles.height, StyleProp::Default) {
                                styles.height = StyleProp::Value(Units::Pixels(measurement.1));
                            }
                        } else {
                            needs_layout = true;
                        }
                    } else {
                        needs_layout = true;
                    }
                } else {
                    needs_layout = true;
                }
            } else {
                needs_layout = true;
            }
        }
        _ => {}
    }

    if needs_layout {
        commands.entity(id.0).insert(DirtyNode);
    }

    (render_primitive, needs_layout)
}

// pub fn build_nodes_tree(context: &mut Context, tree: &Tree, node_query: &Query<(Entity, &Node)>) {
//     if tree.root_node.is_none() {
//         return;
//     }
//     let mut node_tree = Tree::default();
//     node_tree.root_node = tree.root_node;
//     node_tree.children.insert(
//         tree.root_node.unwrap(),
//         get_valid_node_children(&tree, &node_query, tree.root_node.unwrap()),
//     );

//     // let old_focus = self.focus_tree.current();
//     // self.focus_tree.clear();
//     // self.focus_tree.add(root_node_id, &self.tree);

//     for (node_id, node) in node_query.iter() {
//         let node_id = WrappedIndex(node_id);
//         if let Some(widget_styles) = node.raw_styles.as_ref() {
//             // Only add widgets who have renderable nodes.
//             // if widget_styles.render_command.resolve() != RenderCommand::Empty {
//                 let valid_children = get_valid_node_children(&tree, &node_query, node_id);
//                 node_tree.children.insert(node_id, valid_children);
//                 let valid_parent = get_valid_parent(&tree, &node_query, node_id);
//                 if let Some(valid_parent) = valid_parent {
//                     node_tree.parents.insert(node_id, valid_parent);
//                 }
//             // }
//         }

//         // let focusable = self.get_focusable(widget_id).unwrap_or_default();
//         // if focusable {
//         //     self.focus_tree.add(widget_id, &self.tree);
//         // }
//     }

//     // if let Some(old_focus) = old_focus {
//     //     if self.focus_tree.contains(old_focus) {
//     //         self.focus_tree.focus(old_focus);
//     //     }
//     // }

//     // dbg!(&node_tree);

//     // context.node_tree = node_tree;
// }

// pub fn get_valid_node_children(
//     tree: &Tree,
//     query: &Query<(Entity, &Node)>,
//     node_id: WrappedIndex,
// ) -> Vec<WrappedIndex> {
//     let mut children = Vec::new();
//     if let Some(node_children) = tree.children.get(&node_id) {
//         for child_id in node_children {
//             if let Ok((_, _child_node)) = query.get(child_id.0) {
//                 // if child_node.resolved_styles.render_command.resolve() != RenderCommand::Empty {
//                     children.push(*child_id);
//                 // } else {
//                     // children.extend(get_valid_node_children(tree, query, *child_id));
//                 // }
//             } else {
//                 // children.extend(get_valid_node_children(tree, query, *child_id));
//             }
//         }
//     }

//     children
// }

// pub fn get_valid_parent(
//     tree: &Tree,
//     query: &Query<(Entity, &Node)>,
//     node_id: WrappedIndex,
// ) -> Option<WrappedIndex> {
//     if let Some(parent_id) = tree.parents.get(&node_id) {
//         if let Ok((_, parent_node)) = query.get(parent_id.0) {
//             // if parent_node.resolved_styles.render_command.resolve() != RenderCommand::Empty {
//                 return Some(*parent_id);
//             // }
//         }
//         // return get_valid_parent(tree, query, *parent_id);
//     }

//     None
// }
