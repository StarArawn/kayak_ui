// Handles storing more than one item in state..
pub struct MultiState<T> {
    pub data: Vec<T>,
}

impl<T> MultiState<T> {
    pub fn new(first_item: T) -> Self {
        Self {
            data: vec![first_item],
        }
    }

    pub fn get_or_add(&mut self, initial_value: T, index: &mut usize) -> &T {
        if !self.data.get(*index).is_some() {
            self.data.push(initial_value);
        }
        let item = &self.data[*index];
        *index += 1;
        item
    }

    pub fn get(&self, index: usize) -> &T {
        let item = &self.data[index];
        item
    }
}
