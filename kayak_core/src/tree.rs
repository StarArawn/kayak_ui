use std::cmp::Ordering;
use std::iter::Rev;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use morphorm::Hierarchy;

use crate::{Arena, BoxedWidget, Index};

#[derive(Default, Debug, PartialEq)]
pub struct Tree {
    children: HashMap<Index, Vec<Index>>,
    parents: HashMap<Index, Index>,
    root_node: Option<Index>,
    depths: HashMap<Index, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Unchanged,
    Inserted,
    Deleted,
    Updated,
    Moved,
}

#[derive(Default, Debug, Clone)]
pub struct ChildChanges {
    pub changes: Vec<(usize, Index, Index, Vec<Change>)>,
    pub child_changes: Vec<(usize, ChildChanges)>,
}

impl ChildChanges {
    pub fn has_changes(&self) -> bool {
        !self
            .changes
            .iter()
            .all(|change| change.3.iter().all(|c| *c == Change::Unchanged))
    }
}

impl From<Vec<(usize, Index, Index, Vec<Change>)>> for ChildChanges {
    fn from(changes: Vec<(usize, Index, Index, Vec<Change>)>) -> Self {
        Self {
            changes,
            child_changes: Vec::new(),
        }
    }
}

impl Tree {
    pub fn new(root: Index) -> Self {
        let mut tree = Self::default();
        tree.add(root, None);
        tree
    }

    pub fn add(&mut self, index: Index, parent: Option<Index>) {
        if let Some(parent_index) = parent {
            self.parents.insert(index, parent_index);

            let parent_depth = self.depths.get(&parent_index).copied().unwrap_or_default();
            self.depths.insert(index, parent_depth + 1);

            if let Some(parent_children) = self.children.get_mut(&parent_index) {
                parent_children.push(index);
            } else {
                self.children.insert(parent_index, vec![index]);
            }
        } else {
            self.root_node = Some(index);
            self.depths.insert(index, 0);
        }
    }

    /// Add a collection of children to a node in the tree.
    ///
    /// If the node already contains children, the new child nodes will be appended to the list.
    ///
    /// # Panics
    ///
    /// Panics if the given parent node does not already exist within the tree.
    pub fn add_children(&mut self, children: Vec<Index>, parent: Index) {
        assert!(
            self.contains(parent),
            "parent should exist in the tree before adding children"
        );

        let parent_depth = self.depths.get(&parent).copied().unwrap_or_default();
        for child in children.iter() {
            self.parents.insert(*child, parent);
            self.depths.insert(*child, parent_depth + 1);
        }

        if let Some(parent_children) = self.children.get_mut(&parent) {
            parent_children.extend(children);
        } else {
            self.children.insert(parent, children);
        }
    }

    /// Remove the given node and recursively removes its descendants
    pub fn remove(&mut self, index: Index) -> Vec<Index> {
        self.depths.remove(&index);
        let parent = self.parents.remove(&index);
        if let Some(parent) = parent {
            let children = self
                .children
                .remove(&index)
                .unwrap_or_default()
                .into_iter()
                .map(|child| self.remove(child))
                .flatten()
                .collect();
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|node| *node != index);
            }

            children
        } else {
            // Is root node
            self.root_node = None;
            self.parents.clear();
            self.children.clear();
            self.depths.clear();

            Vec::default()
        }
    }

    /// Removes the current node and reparents any children to its current parent.
    ///
    /// Children fill at the original index of the removed node amongst its siblings.
    ///
    /// # Panics
    ///
    /// Panics if called on the root node.
    pub fn remove_and_reparent(&mut self, index: Index) {
        let depth = self.depths.remove(&index).unwrap_or_default();
        let parent = self.parents.remove(&index);
        if let Some(parent) = parent {
            let mut insertion_index = 0usize;

            // === Get Sibling Index === //
            if let Some(siblings) = self.children.get_mut(&parent) {
                insertion_index = siblings.iter().position(|node| *node == index).unwrap();
            }

            // === Reparent Children === //
            if let Some(children) = self.children.remove(&index) {
                for child in children.iter() {
                    self.depths.insert(*child, depth);
                    self.parents.insert(*child, parent);

                    Self::update_depths(&mut self.depths, &self.children, *child, depth);
                }
                if let Some(siblings) = self.children.get_mut(&parent) {
                    siblings.splice(insertion_index..insertion_index + 1, children);
                }
            }
        } else {
            panic!("Cannot reparent a root node's children")
        }
    }

    /// Replace the given node with another, transferring the parent and child relationships over to the replacement node
    pub fn replace(&mut self, index: Index, replace_with: Index) {
        // === Update Parent === //
        if let Some(parent) = self.parents.remove(&index) {
            let depth = self.depths.remove(&index).unwrap_or_default();
            self.depths.insert(replace_with, depth);

            self.parents.insert(replace_with, parent);
            if let Some(siblings) = self.children.get_mut(&parent) {
                let idx = siblings.iter().position(|node| *node == index).unwrap();
                siblings[idx] = replace_with;
            }
        } else {
            self.root_node = Some(replace_with);
            self.depths.remove(&index);
            self.depths.insert(replace_with, 0);
        }

        // === Update Children === //
        if let Some(children) = self.children.remove(&index) {
            for child in children.iter() {
                self.parents.insert(*child, replace_with);
            }
            self.children.insert(replace_with, children);
        }
    }

    /// Directly inserts a parent into the tree.
    ///
    /// This does not run any other checks or additional logic, and should only be used for
    /// intermediate tree mutations— after which, the tree should be corrected to a valid state.
    pub(crate) fn insert_direct(&mut self, index: Index, parent: Option<Index>) {
        if let Some(parent) = parent {
            self.parents.insert(index, parent);
        }
    }

    /// Directly inserts a collection of children into the tree.
    ///
    /// This does not run any other checks or additional logic, and should only be used for
    /// intermediate tree mutations— after which, the tree should be corrected to a valid state.
    pub(crate) fn insert_children_direct(&mut self, children: Vec<Index>, parent: Index) {
        self.children.insert(parent, children);
    }

    /// Recalculates the depths for the tree or subtree of nodes.
    ///
    /// Correct depth mappings are vital for various calculations, including
    /// [`is_descendant`], [`get_common_ancestor`], and [`partial_cmp`]. This is often
    /// properly maintained internally, however, it will often fall into a broken state
    /// if child nodes are inserted into the tree before their parents. Therefore, it's
    /// a good idea to run this method once the tree contains all desired nodes or after
    /// performing a [merge].
    ///
    /// # Arguments
    ///
    /// * `from`: The node from which to recalculate. If `None` the root node is used. If
    ///           a node is given, make sure it has the proper depth set, otherwise, an
    ///           incorrect depth will be propagated to its descendants.
    ///
    /// [`is_descendant`]: Self::is_descendant
    /// [`get_common_ancestor`]: Self::get_common_ancestor
    /// [`partial_cmp`]: Self::partial_cmp
    /// [merge]: Self::merge
    pub fn recalculate_depths(&mut self, from: Option<Index>) {
        if let Some(node) = from.or(self.root_node) {
            let depth = self.depth(node);
            Self::update_depths(&mut self.depths, &self.children, node, depth);
        }
    }

    /// Get the depth of the given node.
    ///
    /// This does _not_ check if the node exists in the tree and will return `0`
    /// by default. If needed, use [`contains`](Self::contains) to check if the
    /// tree contains a node.
    pub fn depth(&self, index: Index) -> usize {
        self.depths.get(&index).copied().unwrap_or_default()
    }

    /// Returns true if the given node is in this tree.
    pub fn contains(&self, index: Index) -> bool {
        Some(index) == self.root_node
            || self.parents.contains_key(&index)
            || self.children.contains_key(&index)
    }

    /// Returns the root node of this tree (if any).
    pub fn root(&self) -> Option<Index> {
        self.root_node
    }

    /// Get the number of nodes in this tree
    pub fn len(&self) -> usize {
        if self.root_node.is_some() {
            self.parents.len() + 1
        } else {
            0
        }
    }

    /// Returns true if this tree has no nodes
    pub fn is_empty(&self) -> bool {
        self.root_node.is_none() && self.parents.is_empty() && self.children.is_empty()
    }

    /// Returns true if the given node is a descendant of another node
    pub fn is_descendant(&self, descendant: Index, of_node: Index) -> bool {
        let descendant_depth = self.depths.get(&descendant);
        let node_depth = self.depths.get(&of_node);
        if let Some(descendant_depth) = descendant_depth {
            if let Some(node_depth) = node_depth {
                if node_depth >= descendant_depth {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            return false;
        }

        let mut index = descendant;
        while let Some(parent) = self.get_parent(index) {
            index = parent;
            if parent == of_node {
                return true;
            }
        }
        false
    }

    /// Compare two nodes.
    ///
    /// Node A is determined to be __greater__ than Node B if any of the following are true:
    /// 1. Node A is a descendant of Node B
    /// 2. Node A is further right than Node B
    ///
    /// In other words, the order in which a node appears during a left-to-right depth-first search
    /// is what determines its [comparative ordering](Ordering).
    ///
    /// Returns `None` if the two nodes cannot be compared, such as when they are not part of the
    /// same tree.
    pub fn partial_cmp(&self, a: Index, b: Index) -> Option<Ordering> {
        if a == b {
            return Some(Ordering::Equal);
        }

        let a_depth = *self.depths.get(&a)?;
        let b_depth = *self.depths.get(&b)?;

        let mut node_a = Some(a);
        let mut node_b = Some(b);

        if a_depth > b_depth {
            // A is deeper than B
            for _ in 0..(a_depth - b_depth) {
                node_a = self.get_parent(node_a?);

                if node_a == Some(b) {
                    // B is ancestor of A
                    return Some(Ordering::Greater);
                }
            }
        } else {
            // B is deeper than A
            for _ in 0..(b_depth - a_depth) {
                node_b = self.get_parent(node_b?);

                if node_b == Some(a) {
                    // A is ancestor of B
                    return Some(Ordering::Less);
                }
            }
        }

        while node_a.is_some() && node_b.is_some() {
            if self.get_parent(node_a.unwrap()) == self.get_parent(node_b.unwrap()) {
                let order_a = self.get_sibling_order(node_a.unwrap());
                let order_b = self.get_sibling_order(node_b.unwrap());
                return order_a.partial_cmp(&order_b);
            }
            node_a = self.get_parent(node_a.unwrap());
            node_b = self.get_parent(node_b.unwrap());
        }

        // Unreachable since we already check that they exist in the same tree (i.e. always share the root node in common)
        unreachable!(
            "nodes `{:?}` and `{:?}` do not share a common ancestor— but definitely should!",
            a, b
        );
    }

    pub fn flatten(&self) -> Vec<Index> {
        if self.root_node.is_none() {
            return Vec::new();
        }

        DownwardIterator::new(&self, Some(self.root_node.unwrap()), true).collect::<Vec<_>>()
    }

    pub fn flatten_node(&self, root_node: Index) -> Vec<Index> {
        if self.root_node.is_none() {
            return Vec::new();
        }

        DownwardIterator::new(&self, Some(root_node), true).collect::<Vec<_>>()
    }

    /// Finds the deepest [_inclusive_] ancestor shared between two nodes.
    ///
    /// If A is a descendant of B, then B is returned. If both A and B are descendants of
    /// C, then C is returned. If A and B are not in the same tree, then `None` is returned.
    ///
    /// [_inclusive_]: https://dom.spec.whatwg.org/#concept-tree-inclusive-ancestor
    pub fn get_common_ancestor(&self, a: Index, b: Index) -> Option<Index> {
        if a == b {
            return Some(a);
        }

        let a_depth = *self.depths.get(&a)?;
        let b_depth = *self.depths.get(&b)?;

        let mut a_parent = self.get_parent(a);
        let mut b_parent = self.get_parent(b);

        if a_depth != b_depth {
            // Match up parent depths
            if a_depth > b_depth {
                // A is deeper than B
                for _ in 0..(a_depth - b_depth) {
                    if a_parent == Some(b) {
                        // B is ancestor of A
                        return a_parent;
                    }
                    a_parent = self.get_parent(a_parent?)
                }
            } else {
                // B is deeper than A
                for _ in 0..(b_depth - a_depth) {
                    if b_parent == Some(a) {
                        // A is ancestor of B
                        return b_parent;
                    }
                    b_parent = self.get_parent(b_parent?)
                }
            }
        }

        while a_parent.is_some() && b_parent.is_some() {
            if a_parent == b_parent {
                return a_parent;
            }
            a_parent = self.get_parent(a_parent.unwrap());
            b_parent = self.get_parent(b_parent.unwrap());
        }

        // Unreachable since we already check that they exist in the same tree (i.e. always share the root node in common)
        unreachable!(
            "nodes `{:?}` and `{:?}` do not share a common ancestor— but definitely should!",
            a, b
        );
    }

    /// Get the parent of the given node.
    ///
    /// Returns `None` if the node is not part of this tree or it is the root element.
    pub fn get_parent(&self, index: Index) -> Option<Index> {
        self.parents
            .get(&index)
            .map_or(None, |parent| Some(*parent))
    }

    /// Get the children of a given node.
    ///
    /// Returns `None` if the node is not part of this tree or it contains no children.
    pub fn get_children(&self, index: Index) -> Option<&[Index]> {
        self.children
            .get(&index)
            .map(|children| children.as_slice())
    }

    /// Returns the total number of children for a given node.
    pub fn child_count(&self, index: Index) -> usize {
        self.children
            .get(&index)
            .map(|children| children.len())
            .unwrap_or_default()
    }

    /// Get the child at an offset within the given node's children.
    ///
    /// Returns `None` if the node is not part of this tree, it contains no children,
    /// or the offset is out of bounds.
    pub fn get_child_at(&self, index: Index, offset: usize) -> Option<Index> {
        self.children
            .get(&index)
            .map(|children| children.get(offset).copied())?
    }

    /// Get the order of a node among its siblings.
    ///
    /// This does _not_ check if the node exists in the tree and will return `0`
    /// by default (or if it is the root element). If needed, use [`contains`](Self::contains)
    /// to check if the tree contains a node.
    pub fn get_sibling_order(&self, index: Index) -> usize {
        if let Some(parent) = self.get_parent(index) {
            self.children
                .get(&parent)
                .map(|children| {
                    children
                        .iter()
                        .position(|child| *child == index)
                        .unwrap_or_default()
                })
                .unwrap_or_default()
        } else {
            0
        }
    }

    pub fn get_first_child(&self, index: Index) -> Option<Index> {
        self.children.get(&index).map_or(None, |children| {
            children
                .first()
                .map_or(None, |first_child| Some(*first_child))
        })
    }

    pub fn get_last_child(&self, index: Index) -> Option<Index> {
        self.children.get(&index).map_or(None, |children| {
            children.last().map_or(None, |last_child| Some(*last_child))
        })
    }

    pub fn get_next_sibling(&self, index: Index) -> Option<Index> {
        let order = self.get_sibling_order(index);
        if let Some(children) = self.get_children(index) {
            children.get(order + 1).map(|sibling| *sibling)
        } else {
            None
        }
    }

    pub fn get_prev_sibling(&self, index: Index) -> Option<Index> {
        let order = self.get_sibling_order(index);
        if order == 0 {
            return None;
        }

        if let Some(children) = self.get_children(index) {
            children.get(order - 1).map(|sibling| *sibling)
        } else {
            None
        }
    }

    pub fn diff_children(&self, other_tree: &Tree, root_node: Index) -> ChildChanges {
        let children_a = self.children.get(&root_node);
        let children_b = other_tree.children.get(&root_node);
        // Handle both easy cases first..
        if children_a.is_some() && children_b.is_none() {
            return children_a
                .unwrap()
                .iter()
                .enumerate()
                .map(|(child_id, child_node)| {
                    (child_id, *child_node, root_node, vec![Change::Deleted])
                })
                .collect::<Vec<_>>()
                .into();
        } else if children_a.is_none() && children_b.is_some() {
            return children_b
                .unwrap()
                .iter()
                .enumerate()
                .map(|(child_id, child_node)| {
                    (child_id, *child_node, root_node, vec![Change::Inserted])
                })
                .collect::<Vec<_>>()
                .into();
        } else if children_a.is_none() && children_b.is_none() {
            return vec![].into();
        }

        let mut child_changes = ChildChanges::default();

        let children_a = children_a
            .unwrap()
            .into_iter()
            .map(|i| *i)
            .enumerate()
            .collect::<Vec<(usize, Index)>>();
        let children_b = children_b
            .unwrap()
            .into_iter()
            .map(|i| *i)
            .enumerate()
            .collect::<Vec<(usize, Index)>>();

        let deleted_nodes = children_a
            .iter()
            // Find matching child
            .filter(|(_, node)| !children_b.iter().any(|(_, node_b)| node == node_b))
            .map(|(id, node)| (*id, *node, root_node, vec![Change::Deleted]))
            .collect::<Vec<_>>();
        child_changes.changes.extend(deleted_nodes);

        let inserted_and_changed = children_b
            .iter()
            .map(|(id, node)| {
                let old_node = children_a.get(*id);
                let inserted =
                    old_node.is_some() && !children_a.iter().any(|(_, old_node)| node == old_node);

                let value_changed = if let Some((_, old_node)) = old_node {
                    node != old_node
                } else {
                    false
                };
                let changed = match (inserted, value_changed) {
                    (true, false) => Change::Inserted,
                    (true, true) => Change::Inserted,
                    (false, true) => Change::Updated,
                    (false, false) => Change::Unchanged,
                };

                (*id, *node, root_node, vec![changed])
            })
            .collect::<Vec<_>>();
        child_changes.changes.extend(inserted_and_changed);

        let flat_tree_diff_nodes = child_changes
            .changes
            .iter()
            .map(|(id, node, parent_node, change)| {
                if change[0] == Change::Deleted {
                    return (0, *node, *parent_node, change.clone());
                } else if change[0] == Change::Inserted {
                    let child_id = other_tree
                        .children
                        .get(parent_node)
                        .unwrap()
                        .iter()
                        .position(|child| child == node)
                        .unwrap();
                    return (child_id, *node, *parent_node, change.clone());
                }

                let parent_a = self.parent(children_a.get(*id).unwrap().1);
                let parent_b = self.parent(*node);
                let definitely_moved = if parent_a.is_some() && parent_b.is_some() {
                    let parent_a = parent_a.unwrap();
                    let parent_b = parent_b.unwrap();
                    parent_a != parent_b
                        || (parent_a == parent_b
                            && *node != children_a.get(*id).unwrap().1
                            && children_a.iter().any(|(_, node_b)| node == node_b))
                } else {
                    false
                };

                if definitely_moved {
                    let change = if change[0] == Change::Unchanged {
                        vec![Change::Moved]
                    } else {
                        if change[0] == Change::Updated {
                            vec![Change::Moved, Change::Updated]
                        } else {
                            vec![Change::Moved]
                        }
                    };
                    return (*id, *node, *parent_node, change);
                }

                (*id, *node, *parent_node, change.clone())
            })
            .collect::<Vec<_>>();
        child_changes.changes = flat_tree_diff_nodes;

        // for (child_id, child_node) in children_a.iter() {
        //     // Add children of child changes.
        //     let children_of_child_changes = self.diff_children(other_tree, *child_node);
        //     child_changes
        //         .child_changes
        //         .push((*child_id, children_of_child_changes));
        // }

        child_changes
    }

    pub fn diff(
        &self,
        other_tree: &Tree,
        root_node: Index,
    ) -> Vec<(usize, Index, Index, Vec<Change>)> {
        let mut changes = Vec::new();

        let mut tree1 = self
            .flatten_node(root_node)
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>();
        let _root_a = tree1.remove(0);
        let mut tree2 = other_tree
            .flatten_node(root_node)
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>();
        let _root_b = tree2.remove(0);

        let deleted_nodes = tree1
            .iter()
            // Find matching child
            .filter(|(_, node)| !tree2.iter().any(|(_, node_b)| node == node_b))
            .map(|(id, node)| {
                (
                    *id - 1,
                    *node,
                    self.get_parent(*node).unwrap(),
                    vec![Change::Deleted],
                )
            })
            .collect::<Vec<_>>();
        changes.extend(deleted_nodes);

        let inserted_and_changed = tree2
            .iter()
            .map(|(id, node)| {
                let old_node = tree1.get(*id - 1);
                let inserted =
                    old_node.is_some() && !tree1.iter().any(|(_, old_node)| node == old_node);

                let value_changed = if let Some((_, old_node)) = old_node {
                    node != old_node
                } else {
                    false
                };
                let changed = match (inserted, value_changed) {
                    (true, false) => Change::Inserted,
                    (true, true) => Change::Inserted,
                    (false, true) => Change::Updated,
                    (false, false) => Change::Unchanged,
                };

                (
                    *id - 1,
                    *node,
                    other_tree.get_parent(*node).unwrap(),
                    vec![changed],
                )
            })
            .collect::<Vec<_>>();
        changes.extend(inserted_and_changed);

        let flat_tree_diff_nodes = changes
            .iter()
            .map(|(id, node, parent_node, change)| {
                if change[0] == Change::Deleted {
                    return (0, *node, *parent_node, change.clone());
                } else if change[0] == Change::Inserted {
                    let child_id = other_tree
                        .children
                        .get(parent_node)
                        .unwrap()
                        .iter()
                        .position(|child| child == node)
                        .unwrap();
                    return (child_id, *node, *parent_node, change.clone());
                }

                let parent_a = self.parent(tree1.get(*id).unwrap().1);
                let parent_b = self.parent(*node);
                let definitely_moved = if parent_a.is_some() && parent_b.is_some() {
                    let parent_a = parent_a.unwrap();
                    let parent_b = parent_b.unwrap();
                    parent_a != parent_b
                        || (parent_a == parent_b
                            && *node != tree1.get(*id).unwrap().1
                            && tree1.iter().any(|(_, node_b)| node == node_b))
                } else {
                    false
                };

                if definitely_moved {
                    let change = if change[0] == Change::Unchanged {
                        vec![Change::Moved]
                    } else {
                        if change[0] == Change::Updated {
                            vec![Change::Moved, Change::Updated]
                        } else {
                            vec![Change::Moved]
                        }
                    };
                    return (*id, *node, *parent_node, change);
                }

                (*id, *node, *parent_node, change.clone())
            })
            .collect::<Vec<_>>();

        flat_tree_diff_nodes
    }

    pub fn merge(&mut self, other: &Tree, root_node: Index, changes: ChildChanges) {
        let has_changes = changes.has_changes();
        let children_a = self.children.get_mut(&root_node);
        let children_b = other.children.get(&root_node);
        if children_a.is_none() && children_b.is_none() {
            // Nothing to do.
            return;
        } else if children_a.is_none() && children_b.is_some() {
            // Simple case of moving all children over to A.
            self.children.insert(root_node, children_b.unwrap().clone());
            return;
        } else if children_a.is_some() && children_b.is_none() {
            // Case for erasing all
            if has_changes {
                let children_a = children_a.unwrap();
                for child in children_a.iter() {
                    self.parents.remove(&*child);
                }
                self.children.remove(&root_node);
            }
            return;
        }
        let children_a = children_a.unwrap();
        let children_b = children_b.unwrap();
        children_a.resize(children_b.len(), Index::default());
        for (id, node, parent_node, change) in changes.changes.iter() {
            match change.as_slice() {
                [Change::Inserted] => {
                    children_a[*id] = *node;
                    self.parents.insert(*node, *parent_node);
                }
                [Change::Moved, Change::Updated] => {
                    children_a[*id] = *node;
                    self.parents.insert(*node, *parent_node);
                }
                [Change::Updated] => {
                    children_a[*id] = *node;
                }
                [Change::Deleted] => {
                    self.parents.remove(node);
                }
                _ => {}
            }
        }

        // for (child_id, children_of_child_changes) in changes.child_changes {
        //     self.merge(
        //         other,
        //         changes.changes[child_id].1,
        //         children_of_child_changes,
        //     );
        // }
    }

    /// Dumps the tree's current state to the console
    ///
    /// To dump only a section of the tree, use [dump_at] instead.
    ///
    /// # Arguments
    ///
    /// * `widgets`: Optionally, provide the current widgets to include metadata about each widget
    ///
    /// returns: ()
    pub fn dump(&self, widgets: Option<&Arena<Option<BoxedWidget>>>) {
        if let Some(root) = self.root_node {
            self.dump_at_internal(root, 0, widgets);
        }
    }

    /// Dumps a section of the tree's current state to the console (starting from a specific index)
    ///
    /// To dump the entire tree, use [dump] instead.
    ///
    /// # Arguments
    ///
    /// * `start_index`: The index to start recursing from (including itself)
    /// * `widgets`: Optionally, provide the current widgets to include metadata about each widget
    ///
    /// returns: ()
    pub fn dump_at(&self, start_index: Index, widgets: Option<&Arena<Option<BoxedWidget>>>) {
        self.dump_at_internal(start_index, 0, widgets);
    }

    fn dump_at_internal(
        &self,
        start_index: Index,
        depth: usize,
        widgets: Option<&Arena<Option<BoxedWidget>>>,
    ) {
        let mut name = None;
        if let Some(widgets) = widgets {
            if let Some(widget) = widgets.get(start_index) {
                if let Some(widget) = widget {
                    name = Some(widget.get_name());
                }
            }
        }

        let indent = "\t".repeat(depth);
        let raw_parts = start_index.into_raw_parts();
        println!(
            "{}{} [{}.{} @ {}]",
            indent,
            name.unwrap_or_default(),
            raw_parts.0,
            raw_parts.1,
            self.depth(start_index)
        );

        if let Some(children) = self.children.get(&start_index) {
            for node_index in children {
                self.dump_at_internal(*node_index, depth + 1, widgets);
            }
        }
    }

    /// Recursively updates the depths of a given node and its children.
    fn update_depths(
        depths: &mut HashMap<Index, usize>,
        children: &HashMap<Index, Vec<Index>>,
        index: Index,
        depth: usize,
    ) {
        depths.insert(index, depth);
        if let Some(childs) = children.get(&index) {
            for child in childs {
                Self::update_depths(depths, children, *child, depth + 1);
            }
        }
    }
}

/// An iterator that performs a depth-first traversal down a tree starting
/// from a given node.
pub struct DownwardIterator<'a> {
    tree: &'a Tree,
    starting_node: Option<Index>,
    current_node: Option<Index>,
    include_self: bool,
}

impl<'a> DownwardIterator<'a> {
    /// Creates a new [`DownwardIterator`] for the given [tree] and [node].
    ///
    /// # Arguments
    ///
    /// * `tree`: The tree to be iterated.
    /// * `starting_node`: The node to start iterating from.
    /// * `include_self`: Whether or not to include the starting node in the output.
    ///
    ///
    /// [tree]: Tree
    /// [node]: Index
    pub fn new(tree: &'a Tree, starting_node: Option<Index>, include_self: bool) -> Self {
        Self {
            tree,
            starting_node,
            current_node: starting_node,
            include_self,
        }
    }
}

impl<'a> Iterator for DownwardIterator<'a> {
    type Item = Index;

    fn next(&mut self) -> Option<Self::Item> {
        if self.include_self {
            self.include_self = false;
            return self.current_node;
        }

        if let Some(current_index) = self.current_node {
            if let Some(first_child) = self.tree.get_first_child(current_index) {
                // Descend!
                self.current_node = Some(first_child);
                return Some(first_child);
            } else if let Some(next_sibling) = self.tree.get_next_sibling(current_index) {
                // Continue from the next sibling
                self.current_node = Some(next_sibling);
                return Some(next_sibling);
            } else if self.current_node == self.starting_node {
                // We've somehow made our way back up to the starting node -> end iteration
                return None;
            } else {
                let mut current_parent = self.tree.get_parent(current_index);
                while current_parent.is_some() {
                    if current_parent == self.starting_node {
                        // Parent is starting node so no need to continue -> end iteration
                        return None;
                    }
                    if let Some(current_parent) = current_parent {
                        if let Some(next_parent_sibling) =
                            self.tree.get_next_sibling(current_parent)
                        {
                            // Continue from the sibling of the parent
                            self.current_node = Some(next_parent_sibling);
                            return Some(next_parent_sibling);
                        }
                    }
                    // Go back up the tree to find the next available node
                    current_parent = self.tree.get_parent(current_parent.unwrap());
                }
            }
        }

        return self.current_node;
    }
}

/// An iterator that performs a single-path traversal up a tree starting
/// from a given node.
pub struct UpwardIterator<'a> {
    tree: &'a Tree,
    current_node: Option<Index>,
    include_self: bool,
}

impl<'a> UpwardIterator<'a> {
    /// Creates a new [`UpwardIterator`] for the given [tree] and [node].
    ///
    /// # Arguments
    ///
    /// * `tree`: The tree to be iterated.
    /// * `starting_node`: The node to start iterating from.
    /// * `include_self`: Whether or not to include the starting node in the output.
    ///
    ///
    /// [tree]: Tree
    /// [node]: Index
    pub fn new(tree: &'a Tree, starting_node: Option<Index>, include_self: bool) -> Self {
        Self {
            tree,
            current_node: starting_node,
            include_self,
        }
    }
}

impl<'a> Iterator for UpwardIterator<'a> {
    type Item = Index;

    fn next(&mut self) -> Option<Self::Item> {
        if self.include_self {
            self.include_self = false;
            return self.current_node;
        }

        self.current_node = self.tree.get_parent(self.current_node?);
        return self.current_node;
    }
}

pub struct ChildIterator<'a> {
    pub tree: &'a Tree,
    pub current_node: Option<Index>,
}

impl<'a> Iterator for ChildIterator<'a> {
    type Item = Index;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entity) = self.current_node {
            self.current_node = self.tree.get_next_sibling(entity);
            return Some(entity);
        }

        None
    }
}

impl<'a> Hierarchy<'a> for Tree {
    type Item = Index;
    type DownIter = DownwardIterator<'a>;
    type UpIter = Rev<std::vec::IntoIter<Index>>;
    type ChildIter = ChildIterator<'a>;

    fn up_iter(&'a self) -> Self::UpIter {
        // We need to convert the downwards iterator into a Vec so that we can reverse it.
        // Morphorm expects the iteration to be the same as Self::DownIter but "in reverse".
        self.flatten().into_iter().rev()
    }

    fn down_iter(&'a self) -> Self::DownIter {
        DownwardIterator::new(self, self.root_node, true)
    }

    fn child_iter(&'a self, node: Self::Item) -> Self::ChildIter {
        let first_child = self.get_first_child(node);
        ChildIterator {
            tree: self,
            current_node: first_child,
        }
    }

    fn parent(&self, node: Self::Item) -> Option<Self::Item> {
        if let Some(parent_index) = self.get_parent(node) {
            return Some(parent_index);
        }

        None
    }

    fn is_first_child(&self, node: Self::Item) -> bool {
        if let Some(parent) = self.parent(node) {
            if let Some(first_child) = self.get_first_child(parent) {
                if first_child == node {
                    return true;
                } else {
                    return false;
                }
            }
        }

        false
    }

    fn is_last_child(&self, node: Self::Item) -> bool {
        if let Some(parent) = self.parent(node) {
            if let Some(parent_children) = self.children.get(&parent) {
                if let Some(last_child) = parent_children.last() {
                    return *last_child == node;
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
pub struct WidgetTree {
    tree: Arc<RwLock<Tree>>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            tree: Arc::new(RwLock::new(Tree::default())),
        }
    }

    pub fn add(&self, index: Index, parent: Option<Index>) {
        if let Ok(mut tree) = self.tree.write() {
            tree.add(index, parent);
        }
    }

    pub fn take(self) -> Tree {
        Arc::try_unwrap(self.tree).unwrap().into_inner().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::node::NodeBuilder;
    use crate::tree::{DownwardIterator, UpwardIterator};
    use crate::{Arena, Index, Tree};
    use std::cmp::Ordering;

    #[test]
    fn test_tree() {
        let mut store = Arena::new();
        let root = store.insert(NodeBuilder::empty().build());
        // Child 1 of root
        let index1 = store.insert(NodeBuilder::empty().build());
        // Children of child 1.
        let index2 = store.insert(NodeBuilder::empty().build());
        let index3 = store.insert(NodeBuilder::empty().build());
        // Child 2 of root
        let index4 = store.insert(NodeBuilder::empty().build());

        let mut tree = Tree::default();
        tree.root_node = Some(root);

        // Setup Parents..
        tree.parents.insert(index1, root);
        tree.parents.insert(index4, root);

        tree.parents.insert(index2, index1);
        tree.parents.insert(index3, index1);

        tree.children.insert(root, vec![index1, index4]);
        tree.children.insert(index1, vec![index2, index3]);

        let flattened = tree.flatten();

        let mapped = flattened
            .iter()
            .map(|x| x.into_raw_parts().0)
            .collect::<Vec<_>>();

        assert!(mapped[0] == 0);
        assert!(mapped[1] == 1);
        assert!(mapped[2] == 2);
        assert!(mapped[3] == 3);
        assert!(mapped[4] == 4);
    }

    #[test]
    fn should_add_children() {
        let mut tree = Tree::default();

        let a = Index::from_raw_parts(0, 0);
        let b = Index::from_raw_parts(1, 0);
        let c = Index::from_raw_parts(2, 0);
        let d = Index::from_raw_parts(3, 0);

        tree.add(a, None);
        tree.add_children(vec![b, c, d], a);

        let mut expected = Tree::default();
        expected.add(a, None);
        expected.add(b, Some(a));
        expected.add(c, Some(a));
        expected.add(d, Some(a));

        assert_eq!(expected, tree);
    }

    #[test]
    fn should_recalculate_depths() {
        let a = Index::from_raw_parts(0, 0);
        let b = Index::from_raw_parts(1, 0);
        let c = Index::from_raw_parts(2, 0);
        let d = Index::from_raw_parts(3, 0);

        let mut expected = Tree::default();
        expected.add(a, None);
        expected.add(b, Some(a));
        expected.add(c, Some(b));
        expected.add(d, Some(c));

        // Test: Reverse-insertion order and recalculate from root
        let mut tree = Tree::default();
        tree.add(c, Some(b));
        tree.add(b, Some(a));
        tree.add(d, Some(c));
        tree.add(a, None);

        assert_eq!(expected.root_node, tree.root_node);
        assert_eq!(expected.parents, tree.parents);
        assert_eq!(expected.children, tree.children);
        assert_ne!(expected.depths, tree.depths);

        tree.recalculate_depths(None);

        assert_eq!(expected.root_node, tree.root_node);
        assert_eq!(expected.parents, tree.parents);
        assert_eq!(expected.children, tree.children);
        assert_eq!(expected.depths, tree.depths);

        // Test: Swapped insertions and recalculate from parent
        let mut tree = Tree::default();
        tree.add(a, None);
        tree.add(b, Some(a));
        tree.add(d, Some(c));
        tree.add(c, Some(b));

        assert_eq!(expected.root_node, tree.root_node);
        assert_eq!(expected.parents, tree.parents);
        assert_eq!(expected.children, tree.children);
        assert_ne!(expected.depths, tree.depths);

        tree.recalculate_depths(Some(b));

        assert_eq!(expected.root_node, tree.root_node);
        assert_eq!(expected.parents, tree.parents);
        assert_eq!(expected.children, tree.children);
        assert_eq!(expected.depths, tree.depths);
    }

    #[test]
    fn should_have_correct_depth() {
        let mut tree = Tree::default();

        // Tree Structure:
        //      A
        //    B   C
        //   D E  F
        //   G

        let a = Index::from_raw_parts(0, 0);
        let b = Index::from_raw_parts(1, 0);
        let c = Index::from_raw_parts(2, 0);
        let d = Index::from_raw_parts(3, 0);
        let e = Index::from_raw_parts(4, 0);
        let f = Index::from_raw_parts(5, 0);
        let g = Index::from_raw_parts(6, 0);

        tree.add(a, None);
        tree.add(b, Some(a));
        tree.add(c, Some(a));
        tree.add(d, Some(b));
        tree.add(e, Some(b));
        tree.add(g, Some(d));
        tree.add(f, Some(c));

        assert_eq!(0, tree.depth(a));
        assert_eq!(1, tree.depth(b));
        assert_eq!(1, tree.depth(c));
        assert_eq!(2, tree.depth(d));
        assert_eq!(2, tree.depth(e));
        assert_eq!(2, tree.depth(f));
        assert_eq!(3, tree.depth(g));
    }

    #[test]
    fn should_descend_tree() {
        let mut tree = Tree::default();

        // Tree Structure:
        //      A
        //    B   C
        //   D E  F
        //   G

        let a = Index::from_raw_parts(0, 0);
        let b = Index::from_raw_parts(1, 0);
        let c = Index::from_raw_parts(2, 0);
        let d = Index::from_raw_parts(3, 0);
        let e = Index::from_raw_parts(4, 0);
        let f = Index::from_raw_parts(5, 0);
        let g = Index::from_raw_parts(6, 0);

        tree.add(a, None);
        tree.add(b, Some(a));
        tree.add(c, Some(a));
        tree.add(d, Some(b));
        tree.add(e, Some(b));
        tree.add(g, Some(d));
        tree.add(f, Some(c));

        macro_rules! assert_descent {
            ($title: literal : $start: ident -> [ $($node: ident),* $(,)? ] ) => {
                let iter = DownwardIterator::new(&tree, Some($start), true);
                let expected_nodes = vec![$start, $($node),*];

                let mut total = 0;
                for (index, node) in iter.enumerate() {
                    let expected = expected_nodes.get(index);
                    assert_eq!(expected, Some(&node), "{} (including self) - expected {:?}, but got {:?}", $title, expected, node);
                    total += 1;
                }
                assert_eq!(expected_nodes.len(), total, "{} (including self) - expected {} nodes, but got {}", $title, expected_nodes.len(), total);

                let iter = DownwardIterator::new(&tree, Some($start), false);
                let expected_nodes = vec![$($node),*];

                let mut total = 0;
                for (index, node) in iter.enumerate() {
                    let expected = expected_nodes.get(index);
                    assert_eq!(expected, Some(&node), "{} (excluding self) - expected {:?}, but got {:?}", $title, expected, node);
                    total += 1;
                }
                assert_eq!(expected_nodes.len(), total, "{} (excluding self) - expected {} nodes, but got {}", $title, expected_nodes.len(), total);
            };

        }

        assert_descent!("A": a -> [b, d, g, e, c, f]);
        assert_descent!("B": b -> [d, g, e]);
        assert_descent!("C": c -> [f]);
        assert_descent!("D": d -> [g]);
        assert_descent!("E": e -> []);
        assert_descent!("F": f -> []);
        assert_descent!("G": g -> []);
    }

    #[test]
    fn should_ascend_tree() {
        let mut tree = Tree::default();

        // Tree Structure:
        //      A
        //    B   C
        //   D E  F
        //   G

        let a = Index::from_raw_parts(0, 0);
        let b = Index::from_raw_parts(1, 0);
        let c = Index::from_raw_parts(2, 0);
        let d = Index::from_raw_parts(3, 0);
        let e = Index::from_raw_parts(4, 0);
        let f = Index::from_raw_parts(5, 0);
        let g = Index::from_raw_parts(6, 0);

        tree.add(a, None);
        tree.add(b, Some(a));
        tree.add(c, Some(a));
        tree.add(d, Some(b));
        tree.add(e, Some(b));
        tree.add(g, Some(d));
        tree.add(f, Some(c));

        macro_rules! assert_ascent {
            ($title: literal : $start: ident -> [ $($node: ident),* $(,)? ] ) => {
                let iter = UpwardIterator::new(&tree, Some($start), true);
                let expected_nodes = vec![$start, $($node),*];

                let mut total = 0;
                for (index, node) in iter.enumerate() {
                    let expected = expected_nodes.get(index);
                    assert_eq!(expected, Some(&node), "{} (including self) - expected {:?}, but got {:?}", $title, expected, node);
                    total += 1;
                }
                assert_eq!(expected_nodes.len(), total, "{} (including self) - expected {} nodes, but got {}", $title, expected_nodes.len(), total);


                let iter = UpwardIterator::new(&tree, Some($start), false);
                let expected_nodes = vec![$($node),*];

                let mut total = 0;
                for (index, node) in iter.enumerate() {
                    let expected = expected_nodes.get(index);
                    assert_eq!(expected, Some(&node), "{} (excluding self) - expected {:?}, but got {:?}", $title, expected, node);
                    total += 1;
                }
                assert_eq!(expected_nodes.len(), total, "{} (excluding self) - expected {} nodes, but got {}", $title, expected_nodes.len(), total);

            };

        }

        assert_ascent!("A": a -> []);
        assert_ascent!("B": b -> [a]);
        assert_ascent!("C": c -> [a]);
        assert_ascent!("D": d -> [b, a]);
        assert_ascent!("E": e -> [b, a]);
        assert_ascent!("F": f -> [c, a]);
        assert_ascent!("G": g -> [d, b, a]);
    }

    #[test]
    fn should_replace() {
        let mut tree = Tree::default();
        let root = Index::from_raw_parts(0, 0);
        let child_a = Index::from_raw_parts(1, 0);
        let child_b = Index::from_raw_parts(2, 0);
        let grandchild_a = Index::from_raw_parts(3, 0);
        let grandchild_b = Index::from_raw_parts(4, 0);
        tree.add(root, None);
        tree.add(child_a, Some(root));
        tree.add(child_b, Some(root));
        tree.add(grandchild_a, Some(child_a));
        tree.add(grandchild_b, Some(child_b));

        let mut expected = Tree::default();
        let expected_root = Index::from_raw_parts(5, 0);
        let expected_child_a = Index::from_raw_parts(6, 0);
        let expected_child_b = Index::from_raw_parts(7, 0);
        let expected_grandchild_a = Index::from_raw_parts(8, 0);
        let expected_grandchild_b = Index::from_raw_parts(9, 0);
        expected.add(expected_root, None);
        expected.add(expected_child_a, Some(expected_root));
        expected.add(expected_child_b, Some(expected_root));
        expected.add(expected_grandchild_a, Some(expected_child_a));
        expected.add(expected_grandchild_b, Some(expected_child_b));

        tree.replace(grandchild_b, expected_grandchild_b);
        assert!(tree
            .children
            .get(&child_b)
            .unwrap()
            .contains(&expected_grandchild_b));
        assert!(!tree.children.get(&child_b).unwrap().contains(&grandchild_b));

        tree.replace(grandchild_a, expected_grandchild_a);
        assert!(tree
            .children
            .get(&child_a)
            .unwrap()
            .contains(&expected_grandchild_a));
        assert!(!tree.children.get(&child_a).unwrap().contains(&grandchild_a));

        tree.replace(child_a, expected_child_a);
        assert!(tree
            .children
            .get(&root)
            .unwrap()
            .contains(&expected_child_a));
        assert!(!tree.children.get(&root).unwrap().contains(&child_a));
        assert_eq!(
            expected_child_a,
            tree.get_parent(expected_grandchild_a).unwrap()
        );

        tree.replace(child_b, expected_child_b);
        assert!(tree
            .children
            .get(&root)
            .unwrap()
            .contains(&expected_child_b));
        assert!(!tree.children.get(&root).unwrap().contains(&child_b));
        assert_eq!(
            expected_child_b,
            tree.get_parent(expected_grandchild_b).unwrap()
        );

        tree.replace(root, expected_root);
        assert_eq!(Some(expected_root), tree.root_node);
        assert_eq!(expected_root, tree.get_parent(expected_child_a).unwrap());
        assert_eq!(expected_root, tree.get_parent(expected_child_b).unwrap());

        assert_eq!(expected, tree);
    }

    #[test]
    fn should_remove() {
        let mut tree = Tree::default();
        let root = Index::from_raw_parts(0, 0);
        let child_a = Index::from_raw_parts(1, 0);
        let child_b = Index::from_raw_parts(2, 0);
        let grandchild_a = Index::from_raw_parts(3, 0);
        let grandchild_b = Index::from_raw_parts(4, 0);
        tree.add(root, None);
        tree.add(child_a, Some(root));
        tree.add(child_b, Some(root));
        tree.add(grandchild_a, Some(child_a));
        tree.add(grandchild_b, Some(child_b));

        let mut expected = Tree::default();
        expected.add(root, None);
        expected.add(child_a, Some(root));
        expected.add(grandchild_a, Some(child_a));

        tree.remove(child_b);

        assert!(!tree.children.get(&root).unwrap().contains(&child_b));
        assert_eq!(expected, tree);
    }

    #[test]
    fn should_remove_root() {
        let mut tree = Tree::default();
        let root = Index::from_raw_parts(0, 0);
        let child_a = Index::from_raw_parts(1, 0);
        let child_b = Index::from_raw_parts(2, 0);
        let grandchild_a = Index::from_raw_parts(3, 0);
        let grandchild_b = Index::from_raw_parts(4, 0);
        tree.add(root, None);
        tree.add(child_a, Some(root));
        tree.add(child_b, Some(root));
        tree.add(grandchild_a, Some(child_a));
        tree.add(grandchild_b, Some(child_b));

        let expected = Tree::default();

        tree.remove(root);

        assert_eq!(None, tree.root_node);
        assert_eq!(expected, tree);
    }

    #[test]
    fn should_remove_and_reparent() {
        let mut tree = Tree::default();
        let root = Index::from_raw_parts(0, 0);
        let child_a = Index::from_raw_parts(1, 0);
        let child_b = Index::from_raw_parts(2, 0);
        let grandchild_a = Index::from_raw_parts(3, 0);
        let grandchild_b = Index::from_raw_parts(4, 0);
        tree.add(root, None);
        tree.add(child_a, Some(root));
        tree.add(child_b, Some(root));
        tree.add(grandchild_a, Some(child_a));
        tree.add(grandchild_b, Some(child_b));

        let mut expected = Tree::default();
        expected.add(root, None);
        expected.add(child_a, Some(root));
        expected.add(grandchild_a, Some(child_a));
        expected.add(grandchild_b, Some(root));

        tree.remove_and_reparent(child_b);

        assert_eq!(root, tree.get_parent(grandchild_b).unwrap());
        assert!(tree.children.get(&root).unwrap().contains(&grandchild_b));
        assert!(!tree.children.get(&root).unwrap().contains(&child_b));
        assert_eq!(expected, tree);
    }

    #[test]
    fn should_contain_root() {
        let mut tree = Tree::default();
        let root = Index::from_raw_parts(0, 0);
        tree.add(root, None);

        assert!(tree.contains(root));
    }

    #[test]
    fn should_contain_child() {
        let mut tree = Tree::default();
        let root = Index::from_raw_parts(0, 0);
        let child = Index::from_raw_parts(1, 0);
        tree.add(root, None);
        tree.add(child, Some(root));

        assert!(tree.contains(root));
        assert!(tree.contains(child));
    }

    #[test]
    fn should_be_empty() {
        let mut tree = Tree::default();
        assert!(tree.is_empty());
        tree.add(Index::default(), None);
        assert!(!tree.is_empty())
    }

    #[test]
    fn should_be_descendant() {
        let mut tree = Tree::default();
        let root = Index::from_raw_parts(0, 0);
        let child = Index::from_raw_parts(1, 0);
        let grandchild = Index::from_raw_parts(2, 0);
        tree.add(root, None);
        tree.add(child, Some(root));
        tree.add(grandchild, Some(child));

        assert!(!tree.is_descendant(root, root));
        assert!(tree.is_descendant(child, root));
        assert!(tree.is_descendant(grandchild, root));
    }

    #[test]
    fn should_give_len() {
        let mut tree = Tree::default();
        let root = Index::from_raw_parts(0, 0);
        let child = Index::from_raw_parts(1, 0);
        let grandchild = Index::from_raw_parts(2, 0);

        assert_eq!(0, tree.len());
        tree.add(root, None);
        assert_eq!(1, tree.len());
        tree.add(child, Some(root));
        assert_eq!(2, tree.len());
        tree.add(grandchild, Some(child));
        assert_eq!(3, tree.len());
    }

    #[test]
    fn should_be_common_ancestor() {
        let mut tree = Tree::default();
        let root = Index::from_raw_parts(0, 0);
        tree.add(root, None);

        // Ancestor subtree
        //      A
        //    B   C
        //   D E  F
        //   G
        //
        //
        let a = Index::from_raw_parts(1, 0);
        let b = Index::from_raw_parts(2, 0);
        let c = Index::from_raw_parts(3, 0);
        let d = Index::from_raw_parts(4, 0);
        let e = Index::from_raw_parts(5, 0);
        let f = Index::from_raw_parts(6, 0);
        let g = Index::from_raw_parts(7, 0);

        tree.add(a, Some(root));
        tree.add(b, Some(a));
        tree.add(c, Some(a));
        tree.add(d, Some(b));
        tree.add(e, Some(b));
        tree.add(g, Some(d));
        tree.add(f, Some(c));

        let common_ancestor = tree.get_common_ancestor(d, e);
        assert_eq!(Some(b), common_ancestor, "D and E should share B in common");
        let common_ancestor = tree.get_common_ancestor(e, d);
        assert_eq!(Some(b), common_ancestor, "E and D should share B in common");

        let common_ancestor = tree.get_common_ancestor(d, g);
        assert_eq!(Some(d), common_ancestor, "D and G should share D in common");
        let common_ancestor = tree.get_common_ancestor(g, d);
        assert_eq!(Some(d), common_ancestor, "G and D should share D in common");

        let common_ancestor = tree.get_common_ancestor(g, f);
        assert_eq!(Some(a), common_ancestor, "G and F should share A in common");
        let common_ancestor = tree.get_common_ancestor(f, g);
        assert_eq!(Some(a), common_ancestor, "F and G should share A in common");

        let common_ancestor = tree.get_common_ancestor(a, a);
        assert_eq!(Some(a), common_ancestor, "A and A should share A in common");
        let common_ancestor = tree.get_common_ancestor(b, b);
        assert_eq!(Some(b), common_ancestor, "B and B should share B in common");

        let z = Index::from_raw_parts(123, 0);
        let common_ancestor = tree.get_common_ancestor(a, z);
        assert_eq!(
            None, common_ancestor,
            "A and Z should share nothing in common"
        );
    }

    #[test]
    fn should_compare_nodes() {
        let mut tree = Tree::default();
        let root = Index::from_raw_parts(0, 0);
        tree.add(root, None);

        // Ancestor subtree
        //      A
        //    B   C
        //   D E  F
        //   G
        //
        //
        let a = Index::from_raw_parts(1, 0);
        let b = Index::from_raw_parts(2, 0);
        let c = Index::from_raw_parts(3, 0);
        let d = Index::from_raw_parts(4, 0);
        let e = Index::from_raw_parts(5, 0);
        let f = Index::from_raw_parts(6, 0);
        let g = Index::from_raw_parts(7, 0);

        tree.add(a, Some(root));
        tree.add(b, Some(a));
        tree.add(c, Some(a));
        tree.add(d, Some(b));
        tree.add(e, Some(b));
        tree.add(g, Some(d));
        tree.add(f, Some(c));

        let ordering = tree.partial_cmp(d, e);
        assert_eq!(Some(Ordering::Less), ordering, "D should be less than E");
        let ordering = tree.partial_cmp(e, d);
        assert_eq!(
            Some(Ordering::Greater),
            ordering,
            "E should be greater than D"
        );

        let ordering = tree.partial_cmp(d, g);
        assert_eq!(Some(Ordering::Less), ordering, "D should be less than G");
        let ordering = tree.partial_cmp(g, d);
        assert_eq!(
            Some(Ordering::Greater),
            ordering,
            "G should be greater than D"
        );

        let ordering = tree.partial_cmp(g, f);
        assert_eq!(Some(Ordering::Less), ordering, "G should be less than F");
        let ordering = tree.partial_cmp(f, g);
        assert_eq!(
            Some(Ordering::Greater),
            ordering,
            "F should be greater than G"
        );

        let ordering = tree.partial_cmp(a, a);
        assert_eq!(Some(Ordering::Equal), ordering, "A should be equal to A");
        let ordering = tree.partial_cmp(b, b);
        assert_eq!(Some(Ordering::Equal), ordering, "B should be equal to B");

        let z = Index::from_raw_parts(123, 0);
        let ordering = tree.partial_cmp(a, z);
        assert_eq!(None, ordering, "A and Z should be incomparable");
    }
}
