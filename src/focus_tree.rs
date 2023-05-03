use std::sync::{Arc, RwLock};

use bevy::{
    prelude::{Component, Entity, Reflect, ReflectComponent, Resource},
    utils::HashMap,
};

use crate::{node::WrappedIndex, prelude::Tree};

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct Focusable;

#[derive(Resource, Debug, Clone, Default)]
pub struct FocusTree {
    tree: Arc<RwLock<Tree>>,
    current_focus: Arc<RwLock<Option<WrappedIndex>>>,
}

/// A struct used to track and calculate widget focusability, based on the following rule:
///
/// > Focusability set by a widget itself will _always_ override focusability set by its parent.
#[derive(Debug, Default)]
pub(crate) struct FocusTracker {
    /// The focusability as set by the parent (i.e. from its props)
    parents: HashMap<WrappedIndex, bool>,
    /// The focusability as set by the widget itself (i.e. from its render function)
    widgets: HashMap<WrappedIndex, bool>,
}

impl FocusTree {
    /// Add the given focusable index to the tree
    pub(crate) fn add(&self, index: WrappedIndex, widget_context: &Tree) {
        // Cases to handle:
        // 1. Tree empty -> insert root node
        // 2. Tree not empty
        //   a. Contains parent -> insert child node
        //   b. Not contains parent -> demote and replace root node
        if let Ok(mut tree) = self.tree.try_write() {
            let mut current_index = index;
            while let Some(parent) = widget_context.get_parent(current_index) {
                current_index = parent;
                if tree.contains(parent) {
                    tree.add(index, Some(parent));
                    return;
                }
            }

            if tree.root_node.is_none() {
                // Set root node
                tree.add(index, None);
                self.focus(index.0);
            }
        }
    }

    /// Remove the given focusable index from the tree
    pub(crate) fn remove(&self, index: WrappedIndex) {
        if let (Ok(mut tree), Ok(mut current_focus)) =
            (self.tree.try_write(), self.current_focus.try_write())
        {
            if *current_focus == Some(index) {
                // Blur
                *current_focus = tree.root_node;
            }

            if tree.root_node == Some(index) {
                tree.remove(index);
            } else {
                tree.remove_and_reparent(index);
            }
        }
    }

    /// Checks if the given index is present in the tree
    pub fn contains(&self, index: Entity) -> bool {
        if let Ok(tree) = self.tree.try_read() {
            tree.contains(WrappedIndex(index))
        } else {
            false
        }
    }

    /// Clear the tree and remove the current focus
    pub(crate) fn clear(&mut self) {
        if let Ok(mut tree) = self.tree.try_write() {
            *tree = Tree::default();
            self.blur();
        }
    }

    /// Set the current focus
    pub fn focus(&self, index: Entity) {
        if let Ok(mut current_focus) = self.current_focus.try_write() {
            *current_focus = Some(WrappedIndex(index));
        }
    }

    /// Remove the current focus
    ///
    /// This returns focus to the root node
    pub fn blur(&self) {
        if let (Ok(tree), Ok(mut current_focus)) =
            (self.tree.try_read(), self.current_focus.try_write())
        {
            *current_focus = tree.root_node;
        }
    }

    /// Get the currently focused index
    pub fn current(&self) -> Option<Entity> {
        if let Ok(current_focus) = self.current_focus.try_read() {
            current_focus.map(|i| i.0)
        } else {
            None
        }
    }

    /// Change focus to the next focusable index
    pub fn next(&self) -> Option<Entity> {
        if let Ok(mut current_focus) = self.current_focus.try_write() {
            *current_focus = self.peek_next(*current_focus);
            current_focus.map(|i| i.0)
        } else {
            None
        }
    }

    /// Change focus to the previous focusable index
    pub fn prev(&self) -> Option<Entity> {
        if let Ok(mut current_focus) = self.current_focus.try_write() {
            *current_focus = self.peek_prev(*current_focus);
            current_focus.map(|i| i.0)
        } else {
            None
        }
    }

    /// Peek the next focusable index without actually changing focus
    pub fn peek_next(&self, current_focus: Option<WrappedIndex>) -> Option<WrappedIndex> {
        if let Ok(tree) = self.tree.try_read() {
            if let Some(index) = current_focus {
                // === Enter Children === //
                if let Some(child) = tree.get_first_child(index) {
                    return Some(child);
                }

                // === Enter Siblings === //
                if let Some(sibling) = tree.get_next_sibling(index) {
                    return Some(sibling);
                }

                // === Go Back Up === //
                let mut next = index;
                while let Some(parent) = tree.get_parent(next) {
                    if let Some(uncle) = tree.get_next_sibling(parent) {
                        return Some(uncle);
                    }
                    next = parent;
                }
            }

            // Default to root node to begin the cycle again
            tree.root_node
        } else {
            None
        }
    }

    /// Peek the previous focusable index without actually changing focus
    pub fn peek_prev(&self, current_focus: Option<WrappedIndex>) -> Option<WrappedIndex> {
        if let Ok(tree) = self.tree.try_read() {
            if let Some(index) = current_focus {
                // === Enter Siblings === //
                if let Some(sibling) = tree.get_prev_sibling(index) {
                    let mut next = sibling;
                    while let Some(child) = tree.get_last_child(next) {
                        next = child;
                    }
                    return Some(next);
                }

                // === Enter Parent === //
                if let Some(parent) = tree.get_parent(index) {
                    return Some(parent);
                }

                // === Go Back Down === //
                let mut next = index;
                while let Some(child) = tree.get_last_child(next) {
                    next = child;
                }

                return Some(next);
            }

            tree.root_node
        } else {
            None
        }
    }
}

impl FocusTracker {
    /// Set the focusability of a widget
    ///
    /// The `is_parent_defined` parameter is important because it dictates how the focusability is stored
    /// and calculated.
    ///
    /// Focusability map:
    /// * `Some(true)` - This widget is focusable
    /// * `Some(false)` - This widget is not focusable
    /// * `None` - This widget can be either focusable or not
    ///
    /// # Arguments
    ///
    /// * `index`: The widget ID
    /// * `focusable`: The focusability of the widget
    /// * `is_parent_defined`: Does this setting come from the parent or the widget itself?
    ///
    /// returns: ()
    pub fn set_focusability(
        &mut self,
        index: WrappedIndex,
        focusable: Option<bool>,
        is_parent_defined: bool,
    ) {
        let map = if is_parent_defined {
            &mut self.parents
        } else {
            &mut self.widgets
        };

        if let Some(focusable) = focusable {
            map.insert(index, focusable);
        } else {
            map.remove(&index);
        }
    }

    /// Get the focusability for the given widget
    ///
    /// # Arguments
    ///
    /// * `index`: The widget ID
    ///
    /// returns: Option<bool>
    pub fn get_focusability(&self, index: WrappedIndex) -> Option<bool> {
        if let Some(focusable) = self.widgets.get(&index) {
            Some(*focusable)
        } else {
            self.parents.get(&index).copied()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::focus_tree::FocusTree;
    use crate::node::WrappedIndex;
    use crate::tree::Tree;
    use bevy::prelude::Entity;

    #[test]
    fn next_should_cycle() {
        let focus_tree = FocusTree::default();
        let mut tree = Tree::default();

        let a = WrappedIndex(Entity::from_raw(0));
        tree.add(a, None);
        let a_a = WrappedIndex(Entity::from_raw(1));
        tree.add(a_a, Some(a));
        let a_b = WrappedIndex(Entity::from_raw(2));
        tree.add(a_b, Some(a));
        let a_a_a = WrappedIndex(Entity::from_raw(3));
        tree.add(a_a_a, Some(a_a));
        let a_a_a_a = WrappedIndex(Entity::from_raw(4));
        tree.add(a_a_a_a, Some(a_a_a));
        let a_a_a_b = WrappedIndex(Entity::from_raw(5));
        tree.add(a_a_a_b, Some(a_a_a));
        let a_b_a = WrappedIndex(Entity::from_raw(6));
        tree.add(a_b_a, Some(a_b));

        focus_tree.add(a, &tree);
        focus_tree.add(a_a, &tree);
        focus_tree.add(a_b, &tree);
        focus_tree.add(a_a_a, &tree);
        focus_tree.add(a_a_a_a, &tree);
        focus_tree.add(a_a_a_b, &tree);
        focus_tree.add(a_b_a, &tree);

        dbg!(&focus_tree);

        assert_eq!(Some(a.0), focus_tree.current());
        assert_eq!(Some(a_a.0), focus_tree.next());
        assert_eq!(Some(a_a_a.0), focus_tree.next());
        assert_eq!(Some(a_a_a_a.0), focus_tree.next());
        assert_eq!(Some(a_a_a_b.0), focus_tree.next());
        assert_eq!(Some(a_b.0), focus_tree.next());
        assert_eq!(Some(a_b_a.0), focus_tree.next());

        assert_eq!(Some(a.0), focus_tree.next());
        assert_eq!(Some(a_a.0), focus_tree.next());

        // etc.
    }

    #[test]
    fn prev_should_cycle() {
        let focus_tree = FocusTree::default();
        let mut tree = Tree::default();

        let a = WrappedIndex(Entity::from_raw(0));
        tree.add(a, None);
        let a_a = WrappedIndex(Entity::from_raw(1));
        tree.add(a_a, Some(a));
        let a_b = WrappedIndex(Entity::from_raw(2));
        tree.add(a_b, Some(a));
        let a_a_a = WrappedIndex(Entity::from_raw(3));
        tree.add(a_a_a, Some(a_a));
        let a_a_a_a = WrappedIndex(Entity::from_raw(4));
        tree.add(a_a_a_a, Some(a_a_a));
        let a_a_a_b = WrappedIndex(Entity::from_raw(5));
        tree.add(a_a_a_b, Some(a_a_a));
        let a_b_a = WrappedIndex(Entity::from_raw(6));
        tree.add(a_b_a, Some(a_b));

        focus_tree.add(a, &tree);
        focus_tree.add(a_a, &tree);
        focus_tree.add(a_b, &tree);
        focus_tree.add(a_a_a, &tree);
        focus_tree.add(a_a_a_a, &tree);
        focus_tree.add(a_a_a_b, &tree);
        focus_tree.add(a_b_a, &tree);

        assert_eq!(Some(a.0), focus_tree.current());
        assert_eq!(Some(a_b_a.0), focus_tree.prev());
        assert_eq!(Some(a_b.0), focus_tree.prev());
        assert_eq!(Some(a_a_a_b.0), focus_tree.prev());
        assert_eq!(Some(a_a_a_a.0), focus_tree.prev());
        assert_eq!(Some(a_a_a.0), focus_tree.prev());
        assert_eq!(Some(a_a.0), focus_tree.prev());

        assert_eq!(Some(a.0), focus_tree.prev());
        assert_eq!(Some(a_b_a.0), focus_tree.prev());

        // etc.
    }
}
