use std::{collections::HashMap, iter::Rev};

use morphorm::Hierarchy;

use crate::{node::NodeIndex, Index};

#[derive(Default, Debug)]
pub struct Tree {
    pub children: HashMap<Index, Vec<Index>>,
    pub parents: HashMap<Index, Index>,
    pub root_node: Index,
}

impl Tree {
    pub fn add(&mut self, _child_index: usize, index: Index, parent: Option<Index>) {
        if let Some(parent_index) = parent {
            self.parents.insert(index, parent_index);
            if let Some(parent_children) = self.children.get_mut(&parent_index) {
                parent_children.push(index);
            } else {
                self.children.insert(parent_index, vec![index]);
            }
        } else {
            self.root_node = index;
        }
    }

    pub fn flatten(&self) -> Vec<NodeIndex> {
        let iterator = DownwardIterator {
            tree: &self,
            current_node: Some(NodeIndex(self.root_node)),
            starting: true,
        };

        iterator.collect::<Vec<_>>()
    }

    pub fn get_parent(&self, index: NodeIndex) -> Option<NodeIndex> {
        self.parents
            .get(&index.0)
            .map_or(None, |parent| Some(NodeIndex(*parent)))
    }

    pub fn get_first_child(&self, index: NodeIndex) -> Option<NodeIndex> {
        self.children.get(&index.0).map_or(None, |children| {
            children
                .first()
                .map_or(None, |first_child| Some(NodeIndex(*first_child)))
        })
    }

    pub fn get_last_child(&self, _index: NodeIndex) -> Option<NodeIndex> {
        todo!()
    }

    pub fn get_next_sibling(&self, index: NodeIndex) -> Option<NodeIndex> {
        if let Some(parent_index) = self.get_parent(index) {
            self.children.get(&parent_index.0).map_or(None, |children| {
                children
                    .iter()
                    .position(|child| *child == index.0)
                    .map_or(None, |child_index| {
                        children
                            .get(child_index + 1)
                            .map_or(None, |next_child| Some(NodeIndex(*next_child)))
                    })
            })
        } else {
            None
        }
    }

    pub fn get_prev_sibling(&self, index: NodeIndex) -> Option<NodeIndex> {
        self.children.get(&index.0).map_or(None, |children| {
            children
                .iter()
                .position(|child| *child == index.0)
                .map_or(None, |child_index| {
                    children
                        .get(child_index - 1)
                        .map_or(None, |next_child| Some(NodeIndex(*next_child)))
                })
        })
    }
}

pub struct DownwardIterator<'a> {
    tree: &'a Tree,
    current_node: Option<NodeIndex>,
    starting: bool,
}

impl<'a> DownwardIterator<'a> {}

impl<'a> Iterator for DownwardIterator<'a> {
    type Item = NodeIndex;
    fn next(&mut self) -> Option<NodeIndex> {
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
    pub current_node: Option<NodeIndex>,
}

impl<'a> Iterator for ChildIterator<'a> {
    type Item = NodeIndex;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entity) = self.current_node {
            self.current_node = self.tree.get_next_sibling(entity);
            return Some(entity);
        }

        None
    }
}

impl<'a> Hierarchy<'a> for Tree {
    type Item = NodeIndex;
    type DownIter = std::vec::IntoIter<NodeIndex>;
    type UpIter = Rev<std::vec::IntoIter<NodeIndex>>;
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
        if let Some(parent_index) = self.parents.get(&node.0) {
            return Some(NodeIndex(*parent_index));
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
            if let Some(parent_children) = self.children.get(&parent.0) {
                if let Some(last_child) = parent_children.last() {
                    return *last_child == node.0;
                }
            }
        }

        false
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
    tree.root_node = root;

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
        .map(|x| x.0.into_raw_parts().0)
        .collect::<Vec<_>>();

    assert!(mapped[0] == 0);
    assert!(mapped[1] == 1);
    assert!(mapped[2] == 2);
    assert!(mapped[3] == 3);
    assert!(mapped[4] == 4);
}
