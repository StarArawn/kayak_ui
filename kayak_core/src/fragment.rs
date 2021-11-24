use derivative::*;

use crate::{context::KayakContext, styles::Style, Index, Widget};

#[derive(Derivative)]
#[derivative(Debug, PartialEq)]
pub struct Fragment {
    pub id: Index,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    pub children: crate::Children,
    pub styles: Option<Style>,
}

impl Widget for Fragment {
    fn get_id(&self) -> Index {
        self.id
    }

    fn set_id(&mut self, id: Index) {
        self.id = id;
    }

    fn get_styles(&self) -> Option<Style> {
        self.styles.clone()
    }

    fn render(&mut self, context: &mut KayakContext) {
        dbg!("Rendering fragment children!");
        if let Some(children) = self.children.as_ref() {
            children(Some(self.get_id()), context);
        }
    }
}
