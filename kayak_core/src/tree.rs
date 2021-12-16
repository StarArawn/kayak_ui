use std::{
    collections::HashMap,
    iter::Rev,
    sync::{Arc, RwLock},
};

use morphorm::Hierarchy;

use crate::Index;

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

    pub fn flatten(&self) -> Vec<Index> {
        if self.root_node.is_none() {
            return Vec::new();
        }
        let iterator = DownwardIterator {
            tree: &self,
            current_node: Some(self.root_node.unwrap()),
            starting: true,
        };

        iterator.collect::<Vec<_>>()
    }

    pub fn flatten_node(&self, root_node: Index) -> Vec<Index> {
        if self.root_node.is_none() {
            return Vec::new();
        }
        let iterator = DownwardIterator {
            tree: &self,
            current_node: Some(root_node),
            starting: true,
        };

        iterator.collect::<Vec<_>>()
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

    pub fn get_last_child(&self, _index: Index) -> Option<Index> {
        todo!()
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
        self.children.get(&index).map_or(None, |children| {
            children
                .iter()
                .position(|child| *child == index)
                .map_or(None, |child_index| {
                    children
                        .get(child_index - 1)
                        .map_or(None, |next_child| Some(*next_child))
                })
        })
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
}

pub struct DownwardIterator<'a> {
    tree: &'a Tree,
    current_node: Option<Index>,
    starting: bool,
}

impl<'a> DownwardIterator<'a> {}

impl<'a> Iterator for DownwardIterator<'a> {
    type Item = Index;
    fn next(&mut self) -> Option<Index> {
        if self.starting {
            self.starting = false;
            return self.current_node;
        }

        if let Some(current_index) = self.current_node {
            if let Some(first_child) = self.tree.get_first_child(current_index) {
                self.current_node = Some(first_child);
                return Some(first_child);
            } else if let Some(next_sibling) = self.tree.get_next_sibling(current_index) {
                self.current_node = Some(next_sibling);
                return Some(next_sibling);
            } else {
                let mut current_parent = self.tree.get_parent(current_index);
                while current_parent.is_some() {
                    if let Some(current_parent) = current_parent {
                        if let Some(next_parent_sibling) =
                            self.tree.get_next_sibling(current_parent)
                        {
                            self.current_node = Some(next_parent_sibling);
                            return Some(next_parent_sibling);
                        }
                    }
                    current_parent = self.tree.get_parent(current_parent.unwrap());
                }
            }
        }

        return None;
    }
}

// pub struct UpwardIterator<'a> {
//     tree: &'a Tree,
//     current_node: Option<NodeIndex>,
// }

// impl<'a> Iterator for UpwardIterator<'a> {
//     type Item = NodeIndex;

//     // TODO - Needs Testing
//     fn next(&mut self) -> Option<NodeIndex> {
//         None
//     }
// }

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
    type DownIter = std::vec::IntoIter<Index>;
    type UpIter = Rev<std::vec::IntoIter<Index>>;
    type ChildIter = ChildIterator<'a>;

    fn up_iter(&'a self) -> Self::UpIter {
        self.flatten().into_iter().rev()
    }

    fn down_iter(&'a self) -> Self::DownIter {
        self.flatten().into_iter()
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

#[test]
fn test_tree() {
    use crate::node::NodeBuilder;
    use crate::Arena;
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
