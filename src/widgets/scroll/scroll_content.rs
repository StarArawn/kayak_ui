use bevy::prelude::{Bundle, Commands, Component, Entity, In, Query, With};

use crate::{
    children::KChildren,
    context::WidgetName,
    layout::GeometryChanged,
    layout::LayoutEvent,
    on_layout::OnLayout,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle, LayoutType, RenderCommand, Units},
    widget::Widget,
};

use super::scroll_context::ScrollContext;

#[derive(Component, Default, PartialEq, Eq, Clone)]
pub struct ScrollContentProps;

impl Widget for ScrollContentProps {}

#[derive(Bundle)]
pub struct ScrollContentBundle {
    pub scroll_content_props: ScrollContentProps,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    pub on_layout: OnLayout,
    pub widget_name: WidgetName,
}

impl Default for ScrollContentBundle {
    fn default() -> Self {
        Self {
            scroll_content_props: Default::default(),
            styles: Default::default(),
            computed_styles: ComputedStyles::default(),
            children: Default::default(),
            on_layout: Default::default(),
            widget_name: ScrollContentProps::default().get_name(),
        }
    }
}

pub fn scroll_content_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<
        (&KStyle, &mut ComputedStyles, &KChildren, &mut OnLayout),
        With<ScrollContentProps>,
    >,
    context_query: Query<&ScrollContext>,
) -> bool {
    if let Ok((styles, mut computed_styles, children, mut on_layout)) = query.get_mut(entity) {
        if let Some(context_entity) = widget_context.get_context_entity::<ScrollContext>(entity) {
            if let Ok(scroll_context) = context_query.get(context_entity) {
                // === OnLayout === //
                *on_layout = OnLayout::new(
                    move |In((event, _entity)): In<(LayoutEvent, Entity)>,
                          mut query: Query<&mut ScrollContext>| {
                        if event.flags.intersects(
                            GeometryChanged::WIDTH_CHANGED | GeometryChanged::HEIGHT_CHANGED,
                        ) {
                            if let Ok(mut scroll) = query.get_mut(context_entity) {
                                scroll.content_width = event.layout.width;
                                scroll.content_height = event.layout.height;
                                let scroll_y = scroll.scroll_y;
                                scroll.set_scroll_y(scroll_y);
                            }
                        }

                        event
                    },
                );

                // === Styles === //
                *computed_styles = KStyle::default()
                    .with_style(KStyle {
                        render_command: RenderCommand::Layout.into(),
                        layout_type: LayoutType::Column.into(),
                        min_width: Units::Pixels(
                            scroll_context.scrollbox_width - scroll_context.pad_x - 10.0,
                        )
                        .into(),
                        min_height: Units::Stretch(
                            scroll_context.scrollbox_height - scroll_context.pad_y,
                        )
                        .into(),
                        width: Units::Auto.into(),
                        height: Units::Auto.into(),
                        ..Default::default()
                    })
                    .with_style(styles)
                    .into();

                children.process(&widget_context, &mut commands, Some(entity));
            }
        }
    }

    true
}
