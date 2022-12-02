use bevy::prelude::{Bundle, Color, Commands, Component, Entity, In, ParamSet, Query};

use crate::{
    children::KChildren,
    context::WidgetName,
    cursor::ScrollUnit,
    event::{Event, EventType},
    event_dispatcher::EventDispatcherContext,
    layout::{GeometryChanged, LayoutEvent},
    on_event::OnEvent,
    on_layout::OnLayout,
    prelude::{constructor, rsx, KayakWidgetContext},
    styles::{ComputedStyles, KPositionType, KStyle, LayoutType, RenderCommand, Units},
    widget::Widget,
    widget_state::WidgetState,
    widgets::{
        scroll::{
            scroll_bar::{ScrollBarBundle, ScrollBarProps},
            scroll_content::ScrollContentBundle,
        },
        ClipBundle, ElementBundle,
    },
};

use super::scroll_context::ScrollContext;

#[derive(Component, Default, Clone, PartialEq)]
pub struct ScrollBoxProps {
    /// If true, always shows scrollbars even when there's nothing to scroll
    ///
    /// Individual scrollbars can still be hidden via [`hide_horizontal`](Self::hide_horizontal)
    /// and [`hide_vertical`](Self::hide_vertical).
    pub always_show_scrollbar: bool,
    /// If true, disables horizontal scrolling
    pub disable_horizontal: bool,
    /// If true, disables vertical scrolling
    pub disable_vertical: bool,
    /// If true, hides the horizontal scrollbar
    pub hide_horizontal: bool,
    /// If true, hides the vertical scrollbar
    pub hide_vertical: bool,
    /// The thickness of the scrollbar
    pub scrollbar_thickness: Option<f32>,
    /// The step to scroll by when `ScrollUnit::Line`
    pub scroll_line: Option<f32>,
    /// The color of the scrollbar thumb
    pub thumb_color: Option<Color>,
    /// The styles of the scrollbar thumb
    pub thumb_styles: Option<KStyle>,
    /// The color of the scrollbar track
    pub track_color: Option<Color>,
    /// The styles of the scrollbar track
    pub track_styles: Option<KStyle>,
}

impl Widget for ScrollBoxProps {}

#[derive(Bundle)]
pub struct ScrollBoxBundle {
    pub scroll_box_props: ScrollBoxProps,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    pub on_layout: OnLayout,
    pub widget_name: WidgetName,
}

impl Default for ScrollBoxBundle {
    fn default() -> Self {
        Self {
            scroll_box_props: Default::default(),
            styles: Default::default(),
            computed_styles: ComputedStyles::default(),
            children: Default::default(),
            on_layout: Default::default(),
            widget_name: ScrollBoxProps::default().get_name(),
        }
    }
}

pub fn scroll_box_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<(
        &ScrollBoxProps,
        &KStyle,
        &mut ComputedStyles,
        &KChildren,
        &mut OnLayout,
    )>,
    mut context_query: ParamSet<(Query<&ScrollContext>, Query<&mut ScrollContext>)>,
) -> bool {
    if let Ok((scroll_box, styles, mut computed_styles, scroll_box_children, mut on_layout)) =
        query.get_mut(entity)
    {
        if let Some(context_entity) = widget_context.get_context_entity::<ScrollContext>(entity) {
            if let Ok(scroll_context) = context_query.p0().get(context_entity).cloned() {
                // === Configuration === //
                let always_show_scrollbar = scroll_box.always_show_scrollbar;
                let disable_horizontal = scroll_box.disable_horizontal;
                let disable_vertical = scroll_box.disable_vertical;
                let hide_horizontal = scroll_box.hide_horizontal;
                let hide_vertical = scroll_box.hide_vertical;
                let scrollbar_thickness = scroll_box.scrollbar_thickness.unwrap_or(10.0);
                let scroll_line = scroll_box.scroll_line.unwrap_or(16.0);
                let thumb_color = scroll_box.thumb_color;
                let thumb_styles = scroll_box.thumb_styles.clone();
                let track_color = scroll_box.track_color;
                let track_styles = scroll_box.track_styles.clone();

                let scroll_x = scroll_context.scroll_x();
                let scroll_y = scroll_context.scroll_y();
                let scrollable_width = scroll_context.scrollable_width();
                let scrollable_height = scroll_context.scrollable_height();

                let hori_thickness = scrollbar_thickness;
                let vert_thickness = scrollbar_thickness;

                let hide_horizontal =
                    hide_horizontal || !always_show_scrollbar && scrollable_width < f32::EPSILON;
                let hide_vertical =
                    hide_vertical || !always_show_scrollbar && scrollable_height < f32::EPSILON;

                let pad_x = if hide_vertical { 0.0 } else { vert_thickness };
                let pad_y = if hide_horizontal { 0.0 } else { hori_thickness };

                if pad_x != scroll_context.pad_x || pad_y != scroll_context.pad_y {
                    if let Ok(mut scroll_context_mut) = context_query.p1().get_mut(context_entity) {
                        scroll_context_mut.pad_x = pad_x;
                        scroll_context_mut.pad_y = pad_y;
                    }
                }

                *on_layout = OnLayout::new(
                    move |In((event, _entity)): In<(LayoutEvent, Entity)>,
                          mut query: Query<&mut ScrollContext>| {
                        if event.flags.intersects(
                            GeometryChanged::WIDTH_CHANGED | GeometryChanged::HEIGHT_CHANGED,
                        ) {
                            if let Ok(mut scroll) = query.get_mut(context_entity) {
                                scroll.scrollbox_width = event.layout.width;
                                scroll.scrollbox_height = event.layout.height;
                            }
                        }

                        event
                    },
                );

                // === Styles === //
                *computed_styles = KStyle::default()
                    .with_style(KStyle {
                        render_command: RenderCommand::Layout.into(),
                        ..Default::default()
                    })
                    .with_style(styles)
                    .with_style(KStyle {
                        width: Units::Stretch(1.0).into(),
                        height: Units::Stretch(1.0).into(),
                        ..Default::default()
                    })
                    .into();

                let hbox_styles = KStyle::default().with_style(KStyle {
                    render_command: RenderCommand::Layout.into(),
                    layout_type: LayoutType::Row.into(),
                    width: Units::Stretch(1.0).into(),
                    ..Default::default()
                });
                let vbox_styles = KStyle::default().with_style(KStyle {
                    render_command: RenderCommand::Layout.into(),
                    layout_type: LayoutType::Column.into(),
                    width: Units::Stretch(1.0).into(),
                    ..Default::default()
                });

                let content_styles = KStyle::default().with_style(KStyle {
                    position_type: KPositionType::SelfDirected.into(),
                    top: Units::Pixels(scroll_y).into(),
                    left: Units::Pixels(scroll_x).into(),
                    ..Default::default()
                });

                let event_handler = OnEvent::new(
                    move |In((event_dispatcher_context, _, mut event, _entity)): In<(
                        EventDispatcherContext,
                        WidgetState,
                        Event,
                        Entity,
                    )>,
                          mut query: Query<&mut ScrollContext>| {
                        if let Ok(mut scroll_context) = query.get_mut(context_entity) {
                            match event.event_type {
                                EventType::Scroll(evt) => {
                                    match evt.delta {
                                        ScrollUnit::Line { x, y } => {
                                            if !disable_horizontal {
                                                scroll_context
                                                    .set_scroll_x(scroll_x - x * scroll_line);
                                            }
                                            if !disable_vertical {
                                                scroll_context
                                                    .set_scroll_y(scroll_y + y * scroll_line);
                                            }
                                        }
                                        ScrollUnit::Pixel { x, y } => {
                                            if !disable_horizontal {
                                                scroll_context.set_scroll_x(scroll_x - x);
                                            }
                                            if !disable_vertical {
                                                scroll_context.set_scroll_y(scroll_y + y);
                                            }
                                        }
                                    }
                                    event.stop_propagation();
                                }
                                _ => {}
                            }
                        }
                        (event_dispatcher_context, event)
                    },
                );

                let parent_id = Some(entity);
                rsx! {
                    <ElementBundle on_event={event_handler} styles={hbox_styles}>
                        <ElementBundle styles={vbox_styles}>
                            <ClipBundle>
                                <ScrollContentBundle
                                    children={scroll_box_children.clone()}
                                    styles={content_styles}
                                />
                            </ClipBundle>
                            {if !hide_horizontal {
                                constructor! {
                                    <ScrollBarBundle
                                        scrollbar_props={ScrollBarProps {
                                            disabled: disable_horizontal,
                                            horizontal: true,
                                            thickness: hori_thickness,
                                            thumb_color,
                                            thumb_styles: thumb_styles.clone(),
                                            track_color,
                                            track_styles: track_styles.clone(),
                                        }}
                                    />
                                }
                            }}
                        </ElementBundle>
                        {if !hide_vertical {
                            constructor! {
                                <ScrollBarBundle
                                    scrollbar_props={ScrollBarProps {
                                        disabled: disable_vertical,
                                        thickness: hori_thickness,
                                        thumb_color,
                                        thumb_styles,
                                        track_color,
                                        track_styles,
                                        ..Default::default()
                                    }}
                                />
                            }
                        }}
                    </ElementBundle>
                };
            }
        }
    }
    true
}
