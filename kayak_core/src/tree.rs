use std::iter::Rev;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use std::collections::HashSet;

use morphorm::Hierarchy;

use crate::{Arena, BoxedWidget, Index};

#[derive(Default, Debug, PartialEq)]
pub struct Tree {
    pub children: HashMap<Index, Vec<Index>>,
    pub parents: HashMap<Index, Index>,
    pub root_node: Option<Index>,
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
    pub fn add(&mut self, index: Index, parent: Option<Index>) {
        if let Some(parent_index) = parent {
            self.parents.insert(index, parent_index);
            if let Some(parent_children) = self.children.get_mut(&parent_index) {
                parent_children.push(index);
            } else {
                self.children.insert(parent_index, vec![index]);
            }
        } else {
            self.root_node = Some(index);
        }
    }

    /// Remove the given node and recursively removes its descendants
    pub fn remove(&mut self, index: Index) -> Vec<Index> {
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

            Vec::default()
        }
    }

    /// Removes the current node and reparents any children to its current parent.
    ///
    /// Children fill at the original index of the removed node amongst its siblings.
    ///
    /// Panics if called on the root node
    pub fn remove_and_reparent(&mut self, index: Index) {
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
                    self.parents.insert(*child, parent);
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
            self.parents.insert(replace_with, parent);
            if let Some(siblings) = self.children.get_mut(&parent) {
                let idx = siblings.iter().position(|node| *node == index).unwrap();
                siblings[idx] = replace_with;
            }
        } else {
            self.root_node = Some(replace_with);
        }

        // === Update Children === //
        if let Some(children) = self.children.remove(&index) {
            for child in children.iter() {
                self.parents.insert(*child, replace_with);
            }
            self.children.insert(replace_with, children);
        }
    }

    /// Returns true if the given node is in this tree
    pub fn contains(&self, index: Index) -> bool {
        Some(index) == self.root_node
            || self.parents.contains_key(&index)
            || self.children.contains_key(&index)
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
        let mut index = descendant;
        while let Some(parent) = self.get_parent(index) {
            index = parent;
            if parent == of_node {
                return true;
            }
        }
        false
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

        // Note: This algorithm could be improved by utilizing an Eulerian Path
        // (see https://en.wikipedia.org/wiki/Euler_tour_technique).
        // However, this can be done when/if this method is found to be a bottleneck.
        // Otherwise, we risk negatively impacting performance as nodes are added/removed
        // fairly frequently.

        let mut a_parent = self.get_parent(a);
        let mut b_parent = self.get_parent(b);

        let mut parents = HashSet::new();
        parents.insert(a);
        parents.insert(b);

        // At each step, this will update the current parent for A and B, then
        // check if either have already been found (if so, then return it). Do this
        // until the root has been reached. If the root is shared, return it,
        // otherwise `None`.
        while a_parent.is_some() || b_parent.is_some() {

            if let Some(a_par) = a_parent {
                let is_new = parents.insert(a_par);
                if !is_new {
                    return Some(a_par);
                }

                a_parent = self.get_parent(a_par);
            }
            if let Some(b_par) = b_parent {
                let is_new = parents.insert(b_par);
                if !is_new {
                    return Some(b_par);
                }

                b_parent = self.get_parent(b_par);
            }
        }

        None
    }

    pub fn get_parent(&self, index: Index) -> Option<Index> {
        self.parents
            .get(&index)
            .map_or(None, |parent| Some(*parent))
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
        if let Some(parent_index) = self.get_parent(index) {
            self.children.get(&parent_index).map_or(None, |children| {
                children
                    .iter()
                    .position(|child| *child == index)
                    .map_or(None, |child_index| {
                        children
                            .get(child_index + 1)
                            .map_or(None, |next_child| Some(*next_child))
                    })
            })
        } else {
            None
        }
    }

    pub fn get_prev_sibling(&self, index: Index) -> Option<Index> {
        if let Some(parent_index) = self.get_parent(index) {
            self.children.get(&parent_index).map_or(None, |children| {
                children
                    .iter()
                    .position(|child| *child == index)
                    .map_or(None, |child_index| {
                        if child_index > 0 {
                            children
                                .get(child_index - 1)
                                .map_or(None, |prev_child| Some(*prev_child))
                        } else {
                            None
                        }
                    })
            })
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
            "{}{} [{}:{}]",
            indent,
            name.unwrap_or_default(),
            raw_parts.0,
            raw_parts.1
        );

        if let Some(children) = self.children.get(&start_index) {
            for node_index in children {
                self.dump_at_internal(*node_index, depth + 1, widgets);
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
        if let Some(parent_index) = self.parents.get(&node) {
            return Some(*parent_index);
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
        assert_eq!(None, common_ancestor, "A and Z should share nothing in common");
    }
}
