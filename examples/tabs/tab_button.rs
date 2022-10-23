use bevy::prelude::{Bundle, Color, Commands, Component, Entity, In, Query};
use kayak_ui::prelude::{
    rsx,
    widgets::{KButtonBundle, TextProps, TextWidgetBundle},
    Event, EventDispatcherContext, EventType, KChildren, KStyle, OnEvent, StyleProp, Units, Widget,
    WidgetContext, WidgetName, WidgetProps, WidgetState,
};

use crate::tab_context::TabContext;

#[derive(Component, Default, PartialEq, Clone)]
pub struct TabButton {
    pub index: usize,
    pub title: String,
}

impl Widget for TabButton {}
impl WidgetProps for TabButton {}

#[derive(Bundle)]
pub struct TabButtonBundle {
    pub tab_button: TabButton,
    pub styles: KStyle,
    pub widget_name: WidgetName,
}

impl Default for TabButtonBundle {
    fn default() -> Self {
        Self {
            tab_button: Default::default(),
            styles: Default::default(),
            widget_name: TabButton::default().get_name(),
        }
    }
}

pub fn tab_button_render(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    query: Query<&TabButton>,
    tab_context_query: Query<&mut TabContext>,
) -> bool {
    if let Ok(tab_button) = query.get(entity) {
        let context_entity = widget_context
            .get_context_entity::<TabContext>(entity)
            .unwrap();
        if let Ok(tab_context) = tab_context_query.get(context_entity) {
            let background_color = if tab_context.current_index == tab_button.index {
                Color::rgba(0.0781, 0.0898, 0.101, 1.0)
            } else {
                Color::rgba(0.0781, 0.0898, 0.101, 0.75)
            };
            let parent_id = Some(entity);

            let button_index = tab_button.index;
            let on_event = OnEvent::new(
                move |In((event_dispatcher_context, _, event, _entity)): In<(
                    EventDispatcherContext,
                    WidgetState,
                    Event,
                    Entity,
                )>,
                      mut query: Query<&mut TabContext>| {
                    match event.event_type {
                        EventType::Click(..) => {
                            if let Ok(mut tab_context) = query.get_mut(context_entity) {
                                tab_context.current_index = button_index;
                            }
                        }
                        _ => {}
                    }
                    (event_dispatcher_context, event)
                },
            );

            rsx! {
                <KButtonBundle
                    on_event={on_event}
                    styles={KStyle {
                        background_color: StyleProp::Value(background_color),
                        height: StyleProp::Value(Units::Pixels(25.0)),
                        ..Default::default()
                    }}>
                    <TextWidgetBundle text={TextProps { content: tab_button.title.clone(), size: 16.0, line_height: Some(16.0), ..Default::default() }} />
                </KButtonBundle>
            }
        }
    }
    true
}
