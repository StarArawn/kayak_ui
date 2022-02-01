use derivative::*;

use crate::{context::KayakContext, context_ref::KayakContextRef, styles::Style, Index, Widget};

#[derive(Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
pub struct Fragment {
    pub id: Index,
    #[derivative(Default(value = "None"))]
    pub styles: Option<Style>,
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub children: crate::Children,
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub on_event: Option<crate::OnEvent>,
}

impl Widget for Fragment {
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
        let parent_id = self.get_id();
        if let Some(children) = self.children.take() {
            let mut context = KayakContextRef::new(&mut context.context, Some(parent_id));
            children(Some(parent_id), &mut context);
        } else {
            return;
        }

        // Note: No need to do anything here with this KayakContextRef.
    }
}
