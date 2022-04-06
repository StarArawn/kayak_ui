use crate::core::{rsx, styles::Style, widget, Bound, Children, MutableBound, WidgetProps};
use kayak_core::render_command::RenderCommand;
use kayak_core::styles::{LayoutType, Units};
use kayak_core::{GeometryChanged, OnLayout};

use super::ScrollContext;

/// Props used by the [`ScrollContent`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub(super) struct ScrollContentProps {
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(Children)]
    pub children: Option<Children>,
    #[prop_field(OnLayout)]
    pub on_layout: Option<OnLayout>,
}

#[widget]
/// A widget that contains the content of a [`ScrollBox`](crate::ScrollBox) widget
///
/// The main purpose of this widget is to calculate the size of its children on render. This
/// is needed by the [`ScrollContext`] in order to function properly.
pub(super) fn ScrollContent(props: ScrollContentProps) {
    // === Scroll === //
    let scroll_ctx = context.create_consumer::<ScrollContext>().unwrap();
    let ScrollContext {
        scrollbox_width,
        scrollbox_height,
        pad_x,
        pad_y,
        ..
    } = scroll_ctx.get();

    // === Layout === //
    props.on_layout = Some(OnLayout::new(move |_, evt| {
        if evt
            .flags
            .intersects(GeometryChanged::WIDTH_CHANGED | GeometryChanged::HEIGHT_CHANGED)
        {
            let mut scroll: ScrollContext = scroll_ctx.get();
            scroll.content_width = evt.layout.width;
            scroll.content_height = evt.layout.height;
            scroll_ctx.set(scroll);
        }
    }));

    // === Styles === //
    props.styles = Some(
        Style::default()
            .with_style(Style {
                render_command: RenderCommand::Layout.into(),
                layout_type: LayoutType::Column.into(),
                min_width: Units::Pixels(scrollbox_width - pad_x).into(),
                min_height: Units::Stretch(scrollbox_height - pad_y).into(),
                width: Units::Auto.into(),
                height: Units::Auto.into(),
                ..Default::default()
            })
            .with_style(&props.styles),
    );

    // === Render === //
    rsx! {
        <>
            {children}
        </>
    }
}
