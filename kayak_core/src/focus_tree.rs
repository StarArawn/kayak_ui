use crate::{Index, Tree};

#[derive(Debug, Default, PartialEq)]
pub struct FocusTree {
    tree: Tree,
    current_focus: Option<Index>,
}

impl FocusTree {
    /// Add the given focusable index to the tree
    pub fn add(&mut self, index: Index, parent: Option<Index>) {
        self.tree.add(index, parent);
    }

    /// Remove the given focusable index from the tree
    pub fn remove(&mut self, index: Index) {
        self.tree.remove(index);
    }

    /// Clear the tree and remove the current focus
    pub fn clear(&mut self) {
        self.tree = Tree::default();
        self.blur();
    }

    /// Set the current focus
    pub fn focus(&mut self, index: Index) {
        self.current_focus = Some(index);
    }

    /// Remove the current focus
    pub fn blur(&mut self) {
        self.current_focus = None;
    }

    /// Get the currently focused index
    pub fn current(&self) -> Option<Index> {
        self.current_focus
    }

    /// Change focus to the next focusable index
    pub fn next(&mut self) -> Option<Index> {
        self.current_focus = self.peek_next();
        self.current_focus
    }

    /// Change focus to the previous focusable index
    pub fn prev(&mut self) -> Option<Index> {
        self.current_focus = self.peek_prev();
        self.current_focus
    }

    /// Peek the next focusable index without actually changing focus
    pub fn peek_next(&self) -> Option<Index> {
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
    pub fn peek_prev(&self) -> Option<Index> {
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
}


#[cfg(test)]
mod tests {
    use crate::focus_tree::FocusTree;
    use crate::Index;

    #[test]
    fn next_should_cycle() {
        let mut focus_tree = FocusTree::default();

        let a = Index::from_raw_parts(0, 0);
        focus_tree.add(a, None);
        let a_a = Index::from_raw_parts(1, 0);
        focus_tree.add(a_a, Some(a));
        let a_b = Index::from_raw_parts(2, 0);
        focus_tree.add(a_b, Some(a));
        let a_a_a = Index::from_raw_parts(3, 0);
        focus_tree.add(a_a_a, Some(a_a));
        let a_a_a_a = Index::from_raw_parts(4, 0);
        focus_tree.add(a_a_a_a, Some(a_a_a));
        let a_a_a_b = Index::from_raw_parts(5, 0);
        focus_tree.add(a_a_a_b, Some(a_a_a));
        let a_b_a = Index::from_raw_parts(6, 0);
        focus_tree.add(a_b_a, Some(a_b));

        assert_eq!(focus_tree.current_focus, None);
        assert_eq!(focus_tree.next(), Some(a));
        assert_eq!(focus_tree.next(), Some(a_a));
        assert_eq!(focus_tree.next(), Some(a_a_a));
        assert_eq!(focus_tree.next(), Some(a_a_a_a));
        assert_eq!(focus_tree.next(), Some(a_a_a_b));
        assert_eq!(focus_tree.next(), Some(a_b));
        assert_eq!(focus_tree.next(), Some(a_b_a));

        assert_eq!(focus_tree.next(), Some(a));
        assert_eq!(focus_tree.next(), Some(a_a));

        // etc.
    }

    #[test]
    fn prev_should_cycle() {
        let mut focus_tree = FocusTree::default();

        let a = Index::from_raw_parts(0, 0);
        focus_tree.add(a, None);
        let a_a = Index::from_raw_parts(1, 0);
        focus_tree.add(a_a, Some(a));
        let a_b = Index::from_raw_parts(2, 0);
        focus_tree.add(a_b, Some(a));
        let a_a_a = Index::from_raw_parts(3, 0);
        focus_tree.add(a_a_a, Some(a_a));
        let a_a_a_a = Index::from_raw_parts(4, 0);
        focus_tree.add(a_a_a_a, Some(a_a_a));
        let a_a_a_b = Index::from_raw_parts(5, 0);
        focus_tree.add(a_a_a_b, Some(a_a_a));
        let a_b_a = Index::from_raw_parts(6, 0);
        focus_tree.add(a_b_a, Some(a_b));

        assert_eq!(focus_tree.current_focus, None);
        assert_eq!(focus_tree.prev(), Some(a));
        assert_eq!(focus_tree.prev(), Some(a_b_a));
        assert_eq!(focus_tree.prev(), Some(a_b));
        assert_eq!(focus_tree.prev(), Some(a_a_a_b));
        assert_eq!(focus_tree.prev(), Some(a_a_a_a));
        assert_eq!(focus_tree.prev(), Some(a_a_a));
        assert_eq!(focus_tree.prev(), Some(a_a));

        assert_eq!(focus_tree.prev(), Some(a));
        assert_eq!(focus_tree.prev(), Some(a_b_a));

        // etc.
    }
}