use crate::core::{rsx, styles::Style, widget, Bound, Children, MutableBound, WidgetProps};

use super::ScrollContext;

/// Props used by the [`ScrollContent`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub(super) struct ScrollContentProps {
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(Children)]
    pub children: Option<Children>,
}

#[widget]
/// A widget that contains the content of a [`ScrollBox`](crate::ScrollBox) widget
///
/// The main purpose of this widget is to calculate the size of its children on render. This
/// is needed by the [`ScrollContext`] in order to function properly.
pub(super) fn ScrollContent(props: ScrollContentProps) {
    // === Scroll === //
    let scroll_ctx = context.create_consumer::<ScrollContext>().unwrap();
    let mut scroll: ScrollContext = scroll_ctx.get();
    let content_width = scroll.content_width();
    let content_height = scroll.content_height();

    // === Layout === //
    let id = self.get_id();
    let child_ids = context.get_valid_children(id);
    let mut width = 0.0000;
    let mut height = 0.0000;
    for child_id in &child_ids {
        if let Some(layout) = context.get_layout(child_id) {
            width += layout.width;
            height += layout.height;
        }
    }

    let should_update_size = width != content_width || height != content_height;
    if should_update_size {
        // Size changed since last render -> Notify provider
        scroll.content_width = width;
        scroll.content_height = height;
        scroll_ctx.set(scroll);
    }

    let children = props.get_children();
    if child_ids.is_empty() && !children.is_none() {
        context.mark_dirty();
    }

    // === Render === //
    rsx! {
        <>
            {children}
        </>
    }
}
