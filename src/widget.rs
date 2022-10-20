use crate::context::WidgetName;

pub trait Widget: Send + Sync {
    fn get_name(&self) -> WidgetName {
        WidgetName(std::any::type_name::<Self>().into())
    }
}
