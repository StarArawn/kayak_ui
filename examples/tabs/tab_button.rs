use bevy::prelude::{Bundle, Color, Commands, Component, Entity, In, Query, Res};
use kayak_ui::{
    prelude::{
        rsx, widgets::KButtonBundle, Corner, EventType, KEvent, KStyle, KayakWidgetContext,
        OnEvent, StyleProp, Units, Widget, WidgetName,
    },
    widgets::KButton,
};

use crate::tab_context::TabContext;

#[derive(Component, Default, PartialEq, Eq, Clone)]
pub struct TabButton {
    pub index: usize,
    pub title: String,
}

impl Widget for TabButton {}

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
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
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
                move |In(_entity): In<Entity>,
                      event: Res<KEvent>,
                      mut query: Query<&mut TabContext>| {
                    if let EventType::Click(..) = event.event_type {
                        if let Ok(mut tab_context) = query.get_mut(context_entity) {
                            tab_context.current_index = button_index;
                        }
                    }
                },
            );

            rsx! {
                <KButtonBundle
                    styles={KStyle {
                        background_color: StyleProp::Value(background_color),
                        border_radius: Corner::all(0.0).into(),
                        height: StyleProp::Value(Units::Pixels(25.0)),
                        ..Default::default()
                    }}
                    button={KButton {
                        text: tab_button.title.clone(),
                    }}
                    on_event={on_event}
                />
            };
        }
    }
    true
}
