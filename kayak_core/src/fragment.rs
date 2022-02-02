use derivative::*;

use crate::{context_ref::KayakContextRef, styles::Style, Index, Widget, WidgetProps, Children, OnEvent};

#[derive(Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
pub struct FragmentProps {
    pub styles: Option<Style>,
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub children: crate::Children,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Fragment {
    pub id: Index,
    props: FragmentProps,
}

impl WidgetProps for FragmentProps {
    fn get_children(&self) -> Children {
        self.children.clone()
    }

    fn get_styles(&self) -> Option<Style> {
        self.styles.clone()
    }

    fn get_on_event(&self) -> Option<OnEvent> {
        None
    }

    fn get_focusable(&self) -> Option<bool> {
        Some(false)
    }
}

impl Widget for Fragment {
    type Props = FragmentProps;

    fn constructor(props: Self::Props) -> Self where Self: Sized {
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
        let parent_id = self.get_id();
        if let Some(children) = self.props.children.take() {
            let mut context = KayakContextRef::new(&mut context.context, Some(parent_id));
            children(Some(parent_id), &mut context);
        } else {
            return;
        }

        // Note: No need to do anything here with this KayakContextRef.
    }
}
