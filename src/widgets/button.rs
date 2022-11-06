use bevy::{
    prelude::{Bundle, Color, Commands, Component, Entity, In, Query},
    window::CursorIcon,
};
use kayak_font::Alignment;
use kayak_ui_macros::rsx;

use crate::{
    context::WidgetName,
    event::{Event, EventType},
    event_dispatcher::EventDispatcherContext,
    on_event::OnEvent,
    prelude::{KChildren, KayakWidgetContext, Units},
    styles::{Corner, Edge, KCursorIcon, KStyle, RenderCommand, StyleProp},
    widget::Widget,
    widget_state::WidgetState,
};

use super::{ElementBundle, TextProps, TextWidgetBundle};

#[derive(Component, PartialEq, Clone, Default)]
pub struct KButton {
    pub text: String,
    pub user_styles: KStyle,
}

/// Default button widget
/// Accepts an OnEvent component
#[derive(Bundle)]
pub struct KButtonBundle {
    pub button: KButton,
    pub styles: KStyle,
    pub on_event: OnEvent,
    pub widget_name: WidgetName,
}

impl Default for KButtonBundle {
    fn default() -> Self {
        Self {
            button: Default::default(),
            styles: Default::default(),
            on_event: Default::default(),
            widget_name: KButton::default().get_name(),
        }
    }
}

impl Widget for KButton {}

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct ButtonState {
    hovering: bool,
}

pub fn button_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<(&KButton, &mut KStyle)>,
    state_query: Query<&ButtonState>,
) -> bool {
    if let Ok((button, mut style)) = query.get_mut(entity) {
        let hover_color = Color::rgba(0.592, 0.627, 0.749, 1.0); //Color::rgba(0.549, 0.666, 0.933, 1.0);
                                                                 // let color = Color::rgba(0.254, 0.270, 0.349, 1.0);
        let state_entity =
            widget_context.use_state(&mut commands, entity, ButtonState { hovering: false });

        if let Ok(state) = state_query.get(state_entity) {
            *style = KStyle::default()
                .with_style(KStyle {
                    render_command: StyleProp::Value(RenderCommand::Quad),
                    ..Default::default()
                })
                .with_style(button.user_styles.clone())
                .with_style(KStyle {
                    background_color: Color::rgba(0.254, 0.270, 0.349, 1.0).into(),
                    border_color: if state.hovering {
                        hover_color.into()
                    } else {
                        Color::rgba(0.254, 0.270, 0.349, 1.0).into()
                    },
                    border: Edge::all(2.0).into(),
                    border_radius: StyleProp::Value(Corner::all(10.0)),
                    height: StyleProp::Value(Units::Pixels(28.0)),
                    width: Units::Stretch(1.0).into(),
                    cursor: StyleProp::Value(KCursorIcon(CursorIcon::Hand)),
                    ..Default::default()
                });

            let on_event = OnEvent::new(
                move |In((event_dispatcher_context, _, mut event, _entity)): In<(
                    EventDispatcherContext,
                    WidgetState,
                    Event,
                    Entity,
                )>,
                      mut query: Query<&mut ButtonState>| {
                    if let Ok(mut button) = query.get_mut(state_entity) {
                        match event.event_type {
                            EventType::MouseIn(..) => {
                                event.stop_propagation();
                                button.hovering = true;
                            }
                            EventType::MouseOut(..) => {
                                button.hovering = false;
                            }
                            _ => {}
                        }
                    }
                    (event_dispatcher_context, event)
                },
            );

            let parent_id = Some(entity);
            rsx! {
                <ElementBundle
                    styles={KStyle {
                        width: Units::Stretch(1.0).into(),
                        height: Units::Stretch(1.0).into(),
                        ..Default::default()
                    }}
                    on_event={on_event}
                >
                    <TextWidgetBundle
                        text={TextProps {
                            alignment: Alignment::Middle,
                            content: button.text.clone(),
                            size: 16.0,
                            user_styles: KStyle {
                                top: Units::Stretch(1.0).into(),
                                bottom: Units::Stretch(1.0).into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }}
                    />
                </ElementBundle>
            }
        }
    }

    true
}
