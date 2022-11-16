use bevy::{
    prelude::{Component, Reflect, ReflectComponent},
    utils::HashMap,
};

use crate::{node::WrappedIndex, prelude::Tree};

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct Focusable;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct FocusTree {
    tree: Tree,
    current_focus: Option<WrappedIndex>,
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
    pub fn add(&mut self, index: WrappedIndex, widget_context: &Tree) {
        // Cases to handle:
        // 1. Tree empty -> insert root node
        // 2. Tree not empty
        //   a. Contains parent -> insert child node
        //   b. Not contains parent -> demote and replace root node

        let mut current_index = index;
        while let Some(parent) = widget_context.get_parent(current_index) {
            current_index = parent;
            if self.contains(parent) {
                self.tree.add(index, Some(parent));
                return;
            }
        }

        if self.tree.root_node.is_none() {
            // Set root node
            self.tree.add(index, None);
            self.focus(index);
        }
    }

    /// Remove the given focusable index from the tree
    pub fn remove(&mut self, index: WrappedIndex) {
        if self.current_focus == Some(index) {
            self.blur();
        }

        if self.tree.root_node == Some(index) {
            self.tree.remove(index);
        } else {
            self.tree.remove_and_reparent(index);
        }
    }

    /// Checks if the given index is present in the tree
    pub fn contains(&self, index: WrappedIndex) -> bool {
        self.tree.contains(index)
    }

    /// Clear the tree and remove the current focus
    pub fn clear(&mut self) {
        self.tree = Tree::default();
        self.blur();
    }

    /// Set the current focus
    pub fn focus(&mut self, index: WrappedIndex) {
        self.current_focus = Some(index);
    }

    /// Remove the current focus
    ///
    /// This returns focus to the root node
    pub fn blur(&mut self) {
        self.current_focus = self.tree.root_node;
    }

    /// Get the currently focused index
    pub fn current(&self) -> Option<WrappedIndex> {
        self.current_focus
    }

    /// Change focus to the next focusable index
    pub fn next(&mut self) -> Option<WrappedIndex> {
        self.current_focus = self.peek_next();
        self.current_focus
    }

    /// Change focus to the previous focusable index
    pub fn prev(&mut self) -> Option<WrappedIndex> {
        self.current_focus = self.peek_prev();
        self.current_focus
    }

    /// Peek the next focusable index without actually changing focus
    pub fn peek_next(&self) -> Option<WrappedIndex> {
        if let Some(index) = self.current_focus {
            // === Enter Children === //
            if let Some(child) = self.tree.get_first_child(index) {
                return Some(child);
            }

            // === Enter Siblings === //
            if let Some(sibling) = self.tree.get_next_sibling(index) {
                return Some(sibling);
            }

            // === Go Back Up === //
            let mut next = index;
            while let Some(parent) = self.tree.get_parent(next) {
                if let Some(uncle) = self.tree.get_next_sibling(parent) {
                    return Some(uncle);
                }
                next = parent;
            }
        }

        // Default to root node to begin the cycle again
        self.tree.root_node
    }

    /// Peek the previous focusable index without actually changing focus
    pub fn peek_prev(&self) -> Option<WrappedIndex> {
        if let Some(index) = self.current_focus {
            // === Enter Siblings === //
            if let Some(sibling) = self.tree.get_prev_sibling(index) {
                let mut next = sibling;
                while let Some(child) = self.tree.get_last_child(next) {
                    next = child;
                }
                return Some(next);
            }

            // === Enter Parent === //
            if let Some(parent) = self.tree.get_parent(index) {
                return Some(parent);
            }

            // === Go Back Down === //
            let mut next = index;
            while let Some(child) = self.tree.get_last_child(next) {
                next = child;
            }

            return Some(next);
        }

        self.tree.root_node
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
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
        let mut focus_tree = FocusTree::default();
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

        assert_eq!(Some(a), focus_tree.current_focus);
        assert_eq!(Some(a_a), focus_tree.next());
        assert_eq!(Some(a_a_a), focus_tree.next());
        assert_eq!(Some(a_a_a_a), focus_tree.next());
        assert_eq!(Some(a_a_a_b), focus_tree.next());
        assert_eq!(Some(a_b), focus_tree.next());
        assert_eq!(Some(a_b_a), focus_tree.next());

        assert_eq!(Some(a), focus_tree.next());
        assert_eq!(Some(a_a), focus_tree.next());

        // etc.
    }

    #[test]
    fn prev_should_cycle() {
        let mut focus_tree = FocusTree::default();
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

        assert_eq!(Some(a), focus_tree.current_focus);
        assert_eq!(Some(a_b_a), focus_tree.prev());
        assert_eq!(Some(a_b), focus_tree.prev());
        assert_eq!(Some(a_a_a_b), focus_tree.prev());
        assert_eq!(Some(a_a_a_a), focus_tree.prev());
        assert_eq!(Some(a_a_a), focus_tree.prev());
        assert_eq!(Some(a_a), focus_tree.prev());

        assert_eq!(Some(a), focus_tree.prev());
        assert_eq!(Some(a_b_a), focus_tree.prev());

        // etc.
    }
}
