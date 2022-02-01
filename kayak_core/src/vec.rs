use derivative::*;

use crate::{context::KayakContext, context_ref::KayakContextRef, styles::Style, Index, Widget};

#[derive(Derivative)]
#[derivative(Debug, PartialEq, Clone, Default)]
pub struct VecTracker<T> {
    pub id: Index,
    #[derivative(Default(value = "None"))]
    pub styles: Option<Style>,
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub children: crate::Children,
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub on_event: Option<crate::OnEvent>,
    pub data: Vec<T>,
}

impl<T> VecTracker<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self {
            data,
            id: Index::default(),
            styles: None,
            children: None,
            on_event: None,
        }
    }
}

impl<T, I> From<I> for VecTracker<T>
where
    I: Iterator<Item = T>,
{
    fn from(iter: I) -> Self {
        Self::new(iter.collect())
    }
}

impl<T> Widget for VecTracker<T>
where
    T: Widget + PartialEq + std::fmt::Debug + Clone + Default,
{
    fn get_id(&self) -> Index {
        self.id
    }

    fn focusable(&self) -> Option<bool> {
        Some(false)
    }

    fn set_id(&mut self, id: Index) {
        self.id = id;
    }

    fn get_styles(&self) -> Option<Style> {
        self.styles.clone()
    }

    fn get_name(&self) -> String {
        String::from("Fragment")
    }

    fn on_event(&mut self, _context: &mut KayakContext, _event: &mut crate::Event) {
        // Do nothing.
    }

    fn render(&mut self, context: &mut KayakContextRef) {
        for (index, item) in self.data.iter().enumerate() {
            context.add_widget(item.clone(), index);
        }

        context.commit();
    }
}
