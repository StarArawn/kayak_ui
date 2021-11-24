use as_any::AsAny;

use crate::{context::KayakContext, styles::Style, Index};

pub trait Widget: std::fmt::Debug + AsAny + Send + Sync {
    fn get_id(&self) -> Index;
    fn set_id(&mut self, id: Index);
    fn get_styles(&self) -> Option<Style>;
    fn render(&mut self, context: &mut KayakContext);
}

impl as_any::Downcast for dyn Widget {}
impl as_any::Downcast for dyn Widget + Send {}
impl as_any::Downcast for dyn Widget + Sync {}
impl as_any::Downcast for dyn Widget + Send + Sync {}
