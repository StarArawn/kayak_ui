use as_any::AsAny;

use crate::{context::KayakContext, context_ref::KayakContextRef, styles::Style, Event, Index, OnEvent, Children};

pub trait Widget: std::fmt::Debug + AsAny + Send + Sync {
    /// Returns whether this widget can be focused or not (or unspecified if `None`)
    fn focusable(&self) -> Option<bool>;
    fn get_id(&self) -> Index;
    fn set_id(&mut self, id: Index);
    fn get_styles(&self) -> Option<Style>;
    fn get_name(&self) -> String;
    fn on_event(&mut self, context: &mut KayakContext, event: &mut Event);
    fn render(&mut self, context: &mut KayakContextRef);
}

impl as_any::Downcast for dyn Widget {}
impl as_any::Downcast for dyn Widget + Send {}
impl as_any::Downcast for dyn Widget + Sync {}
impl as_any::Downcast for dyn Widget + Send + Sync {}

pub trait WidgetProps: std::fmt::Debug {
    fn get_children(&self) -> Option<Children>;
    fn get_styles(&self) -> Option<Style>;
    fn get_on_event(&self) -> Option<OnEvent>;
    fn get_focusable(&self) -> Option<bool>;
}
