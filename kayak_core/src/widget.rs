use as_any::AsAny;
use std::any::Any;

use crate::{context_ref::KayakContextRef, styles::Style, Children, Event, Index, OnEvent};

/// An internal trait that has a blanket implementation over all implementors of [`Widget`]
///
/// This ensures that [`BaseWidget`] can never be implemented manually outside of this crate, even
/// if it is exported out (as long as this one isn't).
pub trait SealedWidget {}

/// The base widget trait, used internally
///
/// You should _never_ implement BaseWidget manually. It is automatically implemented on
/// all implementors of [`Widget`].
pub trait BaseWidget: SealedWidget + std::fmt::Debug + Send + Sync {
    fn constructor<P: WidgetProps>(props: P) -> Self
        where
            Self: Sized;
    fn get_id(&self) -> Index;
    fn set_id(&mut self, id: Index);
    fn get_props(&self) -> &dyn WidgetProps;
    fn get_props_mut(&mut self) -> &mut dyn WidgetProps;
    fn render(&mut self, context: &mut KayakContextRef);
    fn get_name(&self) -> &'static str;
    fn on_event(&mut self, context: &mut KayakContextRef, event: &mut Event);
}

/// The main trait for defining a widget
pub trait Widget: std::fmt::Debug + Clone + Default + PartialEq + AsAny + Send + Sync {
    /// The props associated with this widget
    type Props: WidgetProps + Clone + Default + PartialEq;

    /// Construct the widget with the given props
    fn constructor(props: Self::Props) -> Self
        where
            Self: Sized;

    /// Get this widget's ID
    fn get_id(&self) -> Index;

    /// Set this widget's ID
    ///
    /// This method is used internally. You likely do not (or should not) need to call this yourself.
    fn set_id(&mut self, id: Index);

    /// Get a reference to this widget's props
    fn get_props(&self) -> &Self::Props;

    /// Get a mutable reference to this widget's props
    fn get_props_mut(&mut self) -> &mut Self::Props;

    /// The render function for this widget
    ///
    /// This method will be called whenever the widget needs to be re-rendered. It should build its
    /// own [`WidgetTree`](crate::WidgetTree) using [`KayakContextRef`](crate::KayakContextRef) and finalize
    /// the tree using [`KayakContextRef::commit`](crate::KayakContextRef::commit).
    fn render(&mut self, context: &mut KayakContextRef);

    /// Get the name of this widget
    fn get_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Send an event to this widget
    fn on_event(&mut self, context: &mut KayakContextRef, event: &mut Event) {
        if let Some(on_event) = self.get_props().get_on_event() {
            on_event.try_call(context, event);
        }
    }
}

/// Trait for props passed to a widget
pub trait WidgetProps: std::fmt::Debug + AsAny + Send + Sync {
    /// Gets the children of this widget
    ///
    /// Returns `None` if this widget doesn't contain children
    fn get_children(&self) -> Option<Children>;
    /// Sets the children of this widget
    fn set_children(&mut self, children: Option<Children>);
    /// Gets the custom styles of this widget
    ///
    /// Returns `None` if this widget doesn't have any custom styles
    fn get_styles(&self) -> Option<Style>;
    /// Gets the custom event handler of this widget
    ///
    /// Returns `None` if this widget doesn't contain a custom event handler
    fn get_on_event(&self) -> Option<OnEvent>;
    /// Gets the focusability of this widget
    ///
    /// The meanings of the returned values are:
    ///
    /// | Value         | Description                              |
    /// |---------------|------------------------------------------|
    /// | `Some(true)`  | The widget is focusable                  |
    /// | `Some(false)` | The widget is not focusable              |
    /// | `None`        | The widget's focusability is unspecified |
    ///
    fn get_focusable(&self) -> Option<bool>;
}

/// Automatically implements the `BaseWidget` trait for all implementors of [`Widget`]
impl<T> BaseWidget for T
    where
        T: Widget + Clone + PartialEq + Default,
{
    fn constructor<P: WidgetProps>(props: P) -> Self
        where
            Self: Sized,
    {
        let props: Box<dyn Any> = Box::new(props);
        Widget::constructor(*props.downcast::<<T as Widget>::Props>().unwrap())
    }

    fn get_id(&self) -> Index {
        Widget::get_id(self)
    }

    fn set_id(&mut self, id: Index) {
        Widget::set_id(self, id);
    }

    fn get_props(&self) -> &dyn WidgetProps {
        Widget::get_props(self)
    }

    fn get_props_mut(&mut self) -> &mut dyn WidgetProps {
        Widget::get_props_mut(self)
    }

    fn render(&mut self, context: &mut KayakContextRef) {
        Widget::render(self, context);
    }

    fn get_name(&self) -> &'static str {
        Widget::get_name(self)
    }

    fn on_event(&mut self, context: &mut KayakContextRef, event: &mut Event) {
        Widget::on_event(self, context, event);
    }
}

/// Automatically implements the `SealedWidget` trait for all implementors of [`Widget`]
impl<T> SealedWidget for T where T: Widget {}

/// Implements [`WidgetProps`] for the unit type, allowing for "empty" props to
/// be defined as a simple `()`
///
/// This just reduces the amount of code and imports needed to state that a widget contains
/// no adjustable props (i.e., it's top-level or fully contained).
impl WidgetProps for () {
    fn get_children(&self) -> Option<Children> {
        None
    }

    fn set_children(&mut self, _children: Option<Children>) {}

    fn get_styles(&self) -> Option<Style> {
        None
    }

    fn get_on_event(&self) -> Option<OnEvent> {
        None
    }

    fn get_focusable(&self) -> Option<bool> {
        None
    }
}
