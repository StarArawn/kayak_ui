use crate::{
    styles::{Style, StyleProp},
    Arena, Index,
};

/// A widget node used for building the layout tree
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// The list of children directly under this node
    pub children: Vec<Index>,
    /// The ID of this node's widget
    pub id: Index,
    /// The fully resolved styles for this node
    pub resolved_styles: Style,
    /// The raw styles for this node, before style resolution
    pub raw_styles: Option<Style>,
    /// The z-index of this node, used for controlling layering
    pub z: f32,
}

/// A struct used for building a [`Node`]
pub struct NodeBuilder {
    node: Node,
}

impl NodeBuilder {
    /// Defines a basic node without children, styles, etc.
    pub fn empty() -> Self {
        Self {
            node: Node {
                children: Vec::new(),
                id: Index::default(),
                resolved_styles: Style::default(),
                raw_styles: None,
                z: 0.0,
            },
        }
    }

    /// Defines a node with the given id and styles
    pub fn new(id: Index, styles: Style) -> Self {
        Self {
            node: Node {
                children: Vec::new(),
                id,
                resolved_styles: styles,
                raw_styles: None,
                z: 0.0,
            },
        }
    }

    /// Sets the ID of the node being built
    pub fn with_id(mut self, id: Index) -> Self {
        self.node.id = id;
        self
    }

    /// Sets the children of the node being built
    pub fn with_children(mut self, children: Vec<Index>) -> Self {
        self.node.children.extend(children);
        self
    }

    /// Sets the resolved and raw styles, respectively, of the node being built
    pub fn with_styles(mut self, resolved_styles: Style, raw_styles: Option<Style>) -> Self {
        self.node.resolved_styles = resolved_styles;
        self.node.raw_styles = raw_styles;
        self
    }

    /// Completes and builds the actual [`Node`]
    pub fn build(self) -> Node {
        self.node
    }
}

impl<'a> morphorm::Node<'a> for Index {
    type Data = Arena<Option<Node>>;

    fn layout_type(&self, store: &'_ Self::Data) -> Option<morphorm::LayoutType> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.layout_type {
                    StyleProp::Default => Some(morphorm::LayoutType::default()),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::LayoutType::default()),
                };
            }
        }
        return Some(morphorm::LayoutType::default());
    }

    fn position_type(&self, store: &'_ Self::Data) -> Option<morphorm::PositionType> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.position_type {
                    StyleProp::Default => Some(morphorm::PositionType::default()),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::PositionType::default()),
                };
            }
        }
        return Some(morphorm::PositionType::default());
    }

    fn width(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.width {
                    StyleProp::Default => Some(morphorm::Units::Stretch(1.0)),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Stretch(1.0)),
                };
            }
        }
        return Some(morphorm::Units::Stretch(1.0));
    }

    fn height(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.height {
                    StyleProp::Default => Some(morphorm::Units::Stretch(1.0)),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Stretch(1.0)),
                };
            }
        }
        return Some(morphorm::Units::Stretch(1.0));
    }

    fn min_width(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.min_width {
                    StyleProp::Default => Some(morphorm::Units::Pixels(0.0)),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        Some(morphorm::Units::Auto)
    }

    fn min_height(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.min_height {
                    StyleProp::Default => Some(morphorm::Units::Pixels(0.0)),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        Some(morphorm::Units::Auto)
    }

    fn max_width(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.max_width {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        Some(morphorm::Units::Auto)
    }

    fn max_height(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.max_height {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        Some(morphorm::Units::Auto)
    }

    fn left(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.left {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        return Some(morphorm::Units::Auto);
    }

    fn right(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.right {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        return Some(morphorm::Units::Auto);
    }

    fn top(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.top {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        return Some(morphorm::Units::Auto);
    }

    fn bottom(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.bottom {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        return Some(morphorm::Units::Auto);
    }

    fn min_left(&self, _store: &'_ Self::Data) -> Option<morphorm::Units> {
        Some(morphorm::Units::Auto)
    }

    fn max_left(&self, _store: &'_ Self::Data) -> Option<morphorm::Units> {
        Some(morphorm::Units::Auto)
    }

    fn min_right(&self, _store: &'_ Self::Data) -> Option<morphorm::Units> {
        Some(morphorm::Units::Auto)
    }

    fn max_right(&self, _store: &'_ Self::Data) -> Option<morphorm::Units> {
        Some(morphorm::Units::Auto)
    }

    fn min_top(&self, _store: &'_ Self::Data) -> Option<morphorm::Units> {
        Some(morphorm::Units::Auto)
    }

    fn max_top(&self, _store: &'_ Self::Data) -> Option<morphorm::Units> {
        Some(morphorm::Units::Auto)
    }

    fn min_bottom(&self, _store: &'_ Self::Data) -> Option<morphorm::Units> {
        Some(morphorm::Units::Auto)
    }

    fn max_bottom(&self, _store: &'_ Self::Data) -> Option<morphorm::Units> {
        Some(morphorm::Units::Auto)
    }

    fn child_left(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.padding_left {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        return Some(morphorm::Units::Auto);
    }

    fn child_right(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.padding_right {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        return Some(morphorm::Units::Auto);
    }

    fn child_top(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.padding_top {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        return Some(morphorm::Units::Auto);
    }

    fn child_bottom(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.padding_bottom {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(prop),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        return Some(morphorm::Units::Auto);
    }

    fn row_between(&self, _store: &'_ Self::Data) -> Option<morphorm::Units> {
        Some(morphorm::Units::Auto)
    }

    fn col_between(&self, _store: &'_ Self::Data) -> Option<morphorm::Units> {
        Some(morphorm::Units::Auto)
    }

    fn grid_rows(&self, _store: &'_ Self::Data) -> Option<Vec<morphorm::Units>> {
        Some(vec![])
    }

    fn grid_cols(&self, _store: &'_ Self::Data) -> Option<Vec<morphorm::Units>> {
        Some(vec![])
    }

    fn row_index(&self, _store: &'_ Self::Data) -> Option<usize> {
        Some(0)
    }

    fn col_index(&self, _store: &'_ Self::Data) -> Option<usize> {
        Some(0)
    }

    fn row_span(&self, _store: &'_ Self::Data) -> Option<usize> {
        Some(1)
    }

    fn col_span(&self, _store: &'_ Self::Data) -> Option<usize> {
        Some(1)
    }

    fn border_left(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.border {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(morphorm::Units::Pixels(prop.3)),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        Some(morphorm::Units::Auto)
    }

    fn border_right(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.border {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(morphorm::Units::Pixels(prop.1)),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        Some(morphorm::Units::Auto)
    }

    fn border_top(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.border {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(morphorm::Units::Pixels(prop.0)),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        Some(morphorm::Units::Auto)
    }

    fn border_bottom(&self, store: &'_ Self::Data) -> Option<morphorm::Units> {
        if let Some(node) = store.get(*self) {
            if let Some(node) = node {
                return match node.resolved_styles.border {
                    StyleProp::Default => Some(morphorm::Units::Auto),
                    StyleProp::Value(prop) => Some(morphorm::Units::Pixels(prop.2)),
                    _ => Some(morphorm::Units::Auto),
                };
            }
        }
        Some(morphorm::Units::Auto)
    }
}
