use std::collections::hash_map::Iter;
use std::collections::HashMap;

use morphorm::{Cache, GeometryChanged};

use crate::Index;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rect {
    pub posx: f32,
    pub posy: f32,
    pub width: f32,
    pub height: f32,
    pub z_index: f32,
}

impl Rect {
    pub fn contains(&self, point: &(f32, f32)) -> bool {
        (point.0 >= self.posx && point.0 <= self.posx + self.width)
            && (point.1 >= self.posy && point.1 <= self.posy + self.height)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Space {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Default, Debug)]
pub struct LayoutCache {
    // Computed Outputs
    pub rect: HashMap<Index, Rect>,

    // Intermediate Values
    space: HashMap<Index, Space>,
    size: HashMap<Index, Size>,

    child_width_max: HashMap<Index, f32>,
    child_height_max: HashMap<Index, f32>,
    child_width_sum: HashMap<Index, f32>,
    child_height_sum: HashMap<Index, f32>,

    grid_row_max: HashMap<Index, f32>,
    grid_col_max: HashMap<Index, f32>,

    horizontal_free_space: HashMap<Index, f32>,
    horizontal_stretch_sum: HashMap<Index, f32>,

    vertical_free_space: HashMap<Index, f32>,
    vertical_stretch_sum: HashMap<Index, f32>,

    stack_first_child: HashMap<Index, bool>,
    stack_last_child: HashMap<Index, bool>,

    /// A map of node IDs to their `GeometryChanged` flags
    ///
    /// This should only contain entries for nodes that have _at least one_ flag set.
    /// If a node does not have any flags set, then they should be removed from the map.
    geometry_changed: HashMap<Index, GeometryChanged>,

    visible: HashMap<Index, bool>,
}

impl LayoutCache {
    pub fn add(&mut self, node_index: Index) {
        self.space.insert(node_index, Default::default());

        self.child_width_max.insert(node_index, Default::default());
        self.child_height_max.insert(node_index, Default::default());
        self.child_width_sum.insert(node_index, Default::default());
        self.child_height_sum.insert(node_index, Default::default());

        self.grid_row_max.insert(node_index, Default::default());
        self.grid_col_max.insert(node_index, Default::default());

        self.horizontal_free_space
            .insert(node_index, Default::default());
        self.horizontal_stretch_sum
            .insert(node_index, Default::default());

        self.vertical_free_space
            .insert(node_index, Default::default());
        self.vertical_stretch_sum
            .insert(node_index, Default::default());

        self.stack_first_child
            .insert(node_index, Default::default());
        self.stack_last_child.insert(node_index, Default::default());

        self.size.insert(node_index, Default::default());

        self.visible.insert(node_index, true);
    }

    /// Attempts to initialize the node if it hasn't already been initialized.
    fn try_init(&mut self, node: Index) {
        self.rect.entry(node).or_default();
    }

    /// Returns an iterator over nodes whose layout have been changed since the last update
    pub fn iter_changed(&self) -> Iter<'_, Index, GeometryChanged> {
        self.geometry_changed.iter()
    }
}

impl Cache for LayoutCache {
    type Item = Index;

    fn visible(&self, node: Self::Item) -> bool {
        if let Some(value) = self.visible.get(&node) {
            return *value;
        }

        true
    }

    fn geometry_changed(&self, node: Self::Item) -> GeometryChanged {
        if let Some(geometry_changed) = self.geometry_changed.get(&node) {
            return *geometry_changed;
        }

        GeometryChanged::default()
    }

    fn set_geo_changed(&mut self, node: Self::Item, flag: GeometryChanged, value: bool) {
        // This method is guaranteed to be called by morphorm every layout so we'll attempt to initialize here
        self.try_init(node);

        if value {
            // Setting a flag -> Add entry if it does not already exist
            let geometry_changed = self.geometry_changed.entry(node).or_default();
            geometry_changed.set(flag, value);
        } else {
            // Unsetting a flag -> Don't add entry if it does not exist
            if let Some(geometry_changed) = self.geometry_changed.get_mut(&node) {
                geometry_changed.set(flag, value);

                if geometry_changed.is_empty() {
                    self.geometry_changed.remove(&node);
                }
            }
        }
    }

    fn width(&self, node: Self::Item) -> f32 {
        if let Some(rect) = self.rect.get(&node) {
            return rect.width;
        }

        0.0
    }

    fn height(&self, node: Self::Item) -> f32 {
        if let Some(rect) = self.rect.get(&node) {
            return rect.height;
        }

        0.0
    }

    fn posx(&self, node: Self::Item) -> f32 {
        if let Some(rect) = self.rect.get(&node) {
            return rect.posx;
        }

        0.0
    }

    fn posy(&self, node: Self::Item) -> f32 {
        if let Some(rect) = self.rect.get(&node) {
            return rect.posy;
        }

        0.0
    }

    fn left(&self, node: Self::Item) -> f32 {
        if let Some(space) = self.space.get(&node) {
            return space.left;
        }

        0.0
    }

    fn right(&self, node: Self::Item) -> f32 {
        if let Some(space) = self.space.get(&node) {
            return space.right;
        }

        0.0
    }

    fn top(&self, node: Self::Item) -> f32 {
        if let Some(space) = self.space.get(&node) {
            return space.top;
        }

        0.0
    }

    fn bottom(&self, node: Self::Item) -> f32 {
        if let Some(space) = self.space.get(&node) {
            return space.bottom;
        }

        0.0
    }

    fn new_width(&self, node: Self::Item) -> f32 {
        if let Some(size) = self.size.get(&node) {
            return size.width;
        }

        0.0
    }

    fn new_height(&self, node: Self::Item) -> f32 {
        if let Some(size) = self.size.get(&node) {
            return size.height;
        }

        0.0
    }

    fn child_width_max(&self, node: Self::Item) -> f32 {
        *self.child_width_max.get(&node).unwrap()
    }

    /// Get the computed sum of the widths of the child nodes
    fn child_width_sum(&self, node: Self::Item) -> f32 {
        *self.child_width_sum.get(&node).unwrap()
    }

    /// Get the computed maximum width of the child nodes
    fn child_height_max(&self, node: Self::Item) -> f32 {
        *self.child_height_max.get(&node).unwrap()
    }

    /// Get the computed sum of the widths of the child nodes
    fn child_height_sum(&self, node: Self::Item) -> f32 {
        *self.child_height_sum.get(&node).unwrap()
    }

    /// Get the computed maximum grid row
    fn grid_row_max(&self, node: Self::Item) -> f32 {
        *self.grid_row_max.get(&node).unwrap()
    }

    /// Get the computed maximum grid column
    fn grid_col_max(&self, node: Self::Item) -> f32 {
        *self.grid_col_max.get(&node).unwrap()
    }

    // Setters
    fn set_visible(&mut self, node: Self::Item, value: bool) {
        *self.visible.get_mut(&node).unwrap() = value;
    }

    fn set_child_width_sum(&mut self, node: Self::Item, value: f32) {
        *self.child_width_sum.get_mut(&node).unwrap() = value;
    }

    fn set_child_height_sum(&mut self, node: Self::Item, value: f32) {
        *self.child_height_sum.get_mut(&node).unwrap() = value;
    }

    fn set_child_width_max(&mut self, node: Self::Item, value: f32) {
        *self.child_width_max.get_mut(&node).unwrap() = value;
    }

    fn set_child_height_max(&mut self, node: Self::Item, value: f32) {
        *self.child_height_max.get_mut(&node).unwrap() = value;
    }

    fn horizontal_free_space(&self, node: Self::Item) -> f32 {
        *self.horizontal_free_space.get(&node).unwrap()
    }
    fn set_horizontal_free_space(&mut self, node: Self::Item, value: f32) {
        *self.horizontal_free_space.get_mut(&node).unwrap() = value;
    }
    fn vertical_free_space(&self, node: Self::Item) -> f32 {
        *self.vertical_free_space.get(&node).unwrap()
    }
    fn set_vertical_free_space(&mut self, node: Self::Item, value: f32) {
        *self.vertical_free_space.get_mut(&node).unwrap() = value;
    }

    fn horizontal_stretch_sum(&self, node: Self::Item) -> f32 {
        *self.horizontal_stretch_sum.get(&node).unwrap()
    }
    fn set_horizontal_stretch_sum(&mut self, node: Self::Item, value: f32) {
        *self.horizontal_stretch_sum.get_mut(&node).unwrap() = value;
    }
    fn vertical_stretch_sum(&self, node: Self::Item) -> f32 {
        *self.vertical_stretch_sum.get(&node).unwrap()
    }
    fn set_vertical_stretch_sum(&mut self, node: Self::Item, value: f32) {
        *self.vertical_stretch_sum.get_mut(&node).unwrap() = value;
    }

    fn set_grid_row_max(&mut self, node: Self::Item, value: f32) {
        *self.grid_row_max.get_mut(&node).unwrap() = value;
    }

    fn set_grid_col_max(&mut self, node: Self::Item, value: f32) {
        *self.grid_row_max.get_mut(&node).unwrap() = value;
    }

    fn set_width(&mut self, node: Self::Item, value: f32) {
        let rect = self.rect.entry(node).or_default();
        rect.width = value;
    }
    fn set_height(&mut self, node: Self::Item, value: f32) {
        let rect = self.rect.entry(node).or_default();
        rect.height = value;
    }
    fn set_posx(&mut self, node: Self::Item, value: f32) {
        let rect = self.rect.entry(node).or_default();
        rect.posx = value;
    }
    fn set_posy(&mut self, node: Self::Item, value: f32) {
        let rect = self.rect.entry(node).or_default();
        rect.posy = value;
    }

    fn set_left(&mut self, node: Self::Item, value: f32) {
        if let Some(space) = self.space.get_mut(&node) {
            space.left = value;
        }
    }

    fn set_right(&mut self, node: Self::Item, value: f32) {
        if let Some(space) = self.space.get_mut(&node) {
            space.right = value;
        }
    }

    fn set_top(&mut self, node: Self::Item, value: f32) {
        if let Some(space) = self.space.get_mut(&node) {
            space.top = value;
        }
    }

    fn set_bottom(&mut self, node: Self::Item, value: f32) {
        if let Some(space) = self.space.get_mut(&node) {
            space.bottom = value;
        }
    }

    fn set_new_width(&mut self, node: Self::Item, value: f32) {
        if let Some(size) = self.size.get_mut(&node) {
            size.width = value;
        }
    }

    fn set_new_height(&mut self, node: Self::Item, value: f32) {
        if let Some(size) = self.size.get_mut(&node) {
            size.height = value;
        }
    }

    fn stack_first_child(&self, node: Self::Item) -> bool {
        *self.stack_first_child.get(&node).unwrap()
    }

    fn set_stack_first_child(&mut self, node: Self::Item, value: bool) {
        *self.stack_first_child.get_mut(&node).unwrap() = value;
    }

    fn stack_last_child(&self, node: Self::Item) -> bool {
        *self.stack_last_child.get(&node).unwrap()
    }

    fn set_stack_last_child(&mut self, node: Self::Item, value: bool) {
        *self.stack_last_child.get_mut(&node).unwrap() = value;
    }
}
