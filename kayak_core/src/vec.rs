use derivative::*;

use crate::{context::KayakContext, styles::Style, Index, Widget};

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

    fn render(&mut self, context: &mut KayakContext) {
        let tree = crate::WidgetTree::new();

        for (index, item) in self.data.iter().enumerate() {
            let (should_rerender, child_id) =
                context
                    .widget_manager
                    .create_widget(index, item.clone(), Some(self.get_id()));
            tree.add(child_id, Some(self.get_id()));
            if should_rerender {
                let mut child_widget = context.widget_manager.take(child_id);
                child_widget.render(context);
                context.widget_manager.repossess(child_widget);
            }
        }

        // Consume the widget tree taking the inner value
        let tree = tree.take();

        // Evaluate changes to the tree.
        let changes = context
            .widget_manager
            .tree
            .diff_children(&tree, self.get_id());
        context
            .widget_manager
            .tree
            .merge(&tree, self.get_id(), changes);
    }
}
