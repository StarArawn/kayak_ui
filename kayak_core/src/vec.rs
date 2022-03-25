use crate::{context_ref::KayakContextRef, styles::Style, Children, Index, OnEvent, Widget, WidgetProps, OnLayout};

/// Props used by the [`VecTracker`] widget
#[derive(Default, Debug, PartialEq, Clone)]
pub struct VecTrackerProps<T> {
    /// The data to display in sequence
    ///
    /// The type of [T] should be implement the [`Widget`] trait
    pub data: Vec<T>,
    pub styles: Option<Style>,
    pub children: Option<Children>,
    pub on_event: Option<OnEvent>,
    pub on_layout: Option<OnLayout>,
}

/// A widget that renders a `Vec` of widgets
///
/// # Props
///
/// __Type:__ [`VecTrackerProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `focusable` | ❌        |
///
#[derive(Debug, PartialEq, Clone, Default)]
pub struct VecTracker<T> {
    pub id: Index,
    pub props: VecTrackerProps<T>,
}

impl<T> VecTracker<T> {
    pub fn new(data: Vec<T>) -> Self {
        let props = VecTrackerProps {
            data,
            styles: None,
            children: None,
            on_event: None,
            on_layout: None,
        };

        Self {
            id: Index::default(),
            props,
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

impl<T> WidgetProps for VecTrackerProps<T>
where
    T: Widget,
{
    fn get_children(&self) -> Option<Children> {
        self.children.clone()
    }

    fn set_children(&mut self, children: Option<Children>) {
        self.children = children;
    }

    fn get_styles(&self) -> Option<Style> {
        self.styles.clone()
    }

    fn get_on_event(&self) -> Option<OnEvent> {
        self.on_event.clone()
    }

    fn get_on_layout(&self) -> Option<OnLayout> {
        self.on_layout.clone()
    }

    fn get_focusable(&self) -> Option<bool> {
        Some(false)
    }
}

impl<T> Widget for VecTracker<T>
where
    T: Widget,
{
    type Props = VecTrackerProps<T>;

    fn constructor(props: Self::Props) -> Self
    where
        Self: Sized,
    {
        Self {
            id: Index::default(),
            props,
        }
    }

    fn get_id(&self) -> Index {
        self.id
    }

    fn set_id(&mut self, id: Index) {
        self.id = id;
    }

    fn get_props(&self) -> &Self::Props {
        &self.props
    }

    fn get_props_mut(&mut self) -> &mut Self::Props {
        &mut self.props
    }

    fn render(&mut self, context: &mut KayakContextRef) {
        for (index, item) in self.props.data.iter().enumerate() {
            context.add_widget(item.clone(), index);
        }

        context.commit();
    }
}
