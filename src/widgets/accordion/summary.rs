use bevy::prelude::*;
use kayak_ui_macros::{constructor, rsx};

use crate::{
    children::KChildren,
    context::WidgetName,
    event::{EventType, KEvent},
    event_dispatcher::EventDispatcherContext,
    on_event::OnEvent,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, Corner, Edge, KCursorIcon, KStyle, LayoutType, RenderCommand, Units, StyleProp},
    widget::Widget,
    widget_state::WidgetState,
    widgets::{
        BackgroundBundle, ElementBundle, KSvg, KSvgBundle, Svg,
        EXPAND_LESS_HANDLE, EXPAND_MORE_HANDLE,
    },
};

use super::AccordionContext;

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct AccordionSummary {
    pub index: usize,
}

impl Widget for AccordionSummary {}

#[derive(Bundle, Debug, Clone, PartialEq)]
pub struct AccordionSummaryBundle {
    pub accordion: AccordionSummary,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for AccordionSummaryBundle {
    fn default() -> Self {
        Self {
            accordion: Default::default(),
            styles: Default::default(),
            computed_styles: Default::default(),
            children: Default::default(),
            widget_name: AccordionSummary::default().get_name(),
        }
    }
}

pub fn render(
    In((widget_context, accordion_widget)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<(&AccordionSummary, &KChildren, &KStyle, &mut ComputedStyles)>,
    context_query: Query<&AccordionContext>,
) -> bool {
    if let Ok((accordion, passed_children, styles, mut computed_styles)) =
        query.get_mut(accordion_widget)
    {
        *computed_styles = KStyle::default()
            .with_style(styles)
            .with_style(KStyle {
                render_command: RenderCommand::Layout.into(),
                cursor: KCursorIcon(CursorIcon::Hand).into(),
                width: Units::Stretch(1.0).into(),
                height: Units::Auto.into(),
                ..Default::default()
            })
            .into();

        if let Some(context_entity) =
            widget_context.get_context_entity::<AccordionContext>(accordion_widget)
        {
            if let Ok(context) = context_query.get(context_entity) {
                let current_index = accordion.index;
                let on_event = OnEvent::new(
                    move |In((event_dispatcher_context, _, mut event, _entity)): In<(
                        EventDispatcherContext,
                        WidgetState,
                        KEvent,
                        Entity,
                    )>,
                          mut query: Query<&mut AccordionContext>| {
                        if let Ok(mut context) = query.get_mut(context_entity) {
                            event.stop_propagation();
                            event.prevent_default();
                            match event.event_type {
                                EventType::Click(..) => {
                                    context.toggle_current(current_index);
                                }
                                _ => {}
                            }
                        }

                        (event_dispatcher_context, event)
                    },
                );

                let parent_id = Some(accordion_widget);
                rsx! {
                    <BackgroundBundle
                        styles={KStyle {
                            background_color: Color::rgba(0.133, 0.145, 0.2, 1.0).into(),
                            border_radius: if accordion.index > 0 { StyleProp::Unset } else { Corner::new(5.0, 5.0, 0.0, 0.0).into() },
                            width: Units::Stretch(1.0).into(),
                            height: Units::Auto.into(),
                            padding: Edge::new(Units::Pixels(12.0), Units::Pixels(16.0), Units::Pixels(16.0), Units::Pixels(16.0)).into(),
                            layout_type: LayoutType::Row.into(),
                            cursor: KCursorIcon(CursorIcon::Hand).into(),
                            ..Default::default()
                        }}
                        on_event={on_event}
                    >
                        <ElementBundle children={passed_children.clone()} />
                        <ElementBundle
                            styles={KStyle {
                                width: Units::Pixels(25.0).into(),
                                height: Units::Pixels(20.0).into(),
                                cursor: KCursorIcon(CursorIcon::Hand).into(),
                                ..Default::default()
                            }}
                        >
                            {
                                if context.is_open(accordion.index) {
                                    constructor! {
                                        <KSvgBundle
                                            styles={KStyle {
                                                background_color: Color::WHITE.into(),
                                                cursor: KCursorIcon(CursorIcon::Hand).into(),
                                                width: Units::Pixels(35.0).into(),
                                                height: Units::Pixels(30.0).into(),
                                                top: Units::Pixels(-10.0).into(),
                                                ..Default::default()
                                            }}
                                            svg={KSvg(EXPAND_LESS_HANDLE.typed::<Svg>())}
                                        />
                                    }
                                } else {
                                    constructor! {
                                        <KSvgBundle
                                            styles={KStyle {
                                                background_color: Color::WHITE.into(),
                                                cursor: KCursorIcon(CursorIcon::Hand).into(),
                                                width: Units::Pixels(35.0).into(),
                                                height: Units::Pixels(30.0).into(),
                                                top: Units::Pixels(-10.0).into(),
                                                ..Default::default()
                                            }}
                                            svg={KSvg(EXPAND_MORE_HANDLE.typed::<Svg>())}
                                        />
                                    }
                                }
                            }
                        </ElementBundle>
                    </BackgroundBundle>
                };
            }
        }
    }

    true
}
