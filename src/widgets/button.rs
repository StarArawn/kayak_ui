use bevy::{prelude::*, window::CursorIcon};
use kayak_font::Alignment;
use kayak_ui_macros::rsx;

use crate::{
    context::WidgetName,
    event::{EventType, KEvent},
    on_event::OnEvent,
    prelude::{KChildren, KayakWidgetContext, Units},
    styles::{ComputedStyles, Corner, Edge, KCursorIcon, KStyle, RenderCommand, StyleProp},
    widget::Widget,
};

use super::{ElementBundle, TextProps, TextWidgetBundle};

#[derive(Component, PartialEq, Clone, Default)]
pub struct KButton {
    pub text: String,
}

/// Default button widget
/// Accepts an OnEvent component
#[derive(Bundle)]
pub struct KButtonBundle {
    pub button: KButton,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub on_event: OnEvent,
    pub widget_name: WidgetName,
}

impl Default for KButtonBundle {
    fn default() -> Self {
        Self {
            button: Default::default(),
            styles: Default::default(),
            computed_styles: Default::default(),
            on_event: Default::default(),
            widget_name: KButton::default().get_name(),
        }
    }
}

impl Widget for KButton {}

#[derive(Component, Default, Debug, Clone, PartialEq, Eq)]
pub struct ButtonState {
    pub hovering: bool,
}

pub fn button_render(
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    mut query: Query<(&KButton, &KStyle, &mut ComputedStyles)>,
    state_query: Query<&ButtonState>,
) -> bool {
    if let Ok((button, styles, mut computed_styles)) = query.get_mut(entity) {
        let hover_color = Color::rgba(0.592, 0.627, 0.749, 1.0); //Color::rgba(0.549, 0.666, 0.933, 1.0);

        let font_size = styles.font_size.resolve_or(16.);
        let height = styles.height.resolve_or(Units::Pixels(28.));

        let state_entity =
            widget_context.use_state(&mut commands, entity, ButtonState { hovering: false });

        if let Ok(state) = state_query.get(state_entity) {
            *computed_styles = KStyle::default()
                .with_style(KStyle {
                    render_command: StyleProp::Value(RenderCommand::Quad),
                    ..Default::default()
                })
                .with_style(styles)
                .with_style(KStyle {
                    background_color: Color::rgba(0.254, 0.270, 0.349, 1.0).into(),
                    border_color: if state.hovering {
                        hover_color.into()
                    } else {
                        Color::rgba(0.254, 0.270, 0.349, 1.0).into()
                    },
                    border: Edge::all(2.0).into(),
                    border_radius: StyleProp::Value(Corner::all(10.0)),
                    font_size: StyleProp::Value(font_size).into(),
                    height: StyleProp::Value(height),
                    width: Units::Stretch(1.0).into(),
                    cursor: StyleProp::Value(KCursorIcon(CursorIcon::Hand)),
                    ..Default::default()
                })
                .into();

            let on_event = OnEvent::new(
                move |In(_entity): In<Entity>,
                      mut event: ResMut<KEvent>,
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
                        styles={KStyle {
                            top: Units::Stretch(1.0).into(),
                            bottom: Units::Stretch(1.0).into(),
                            left: Units::Stretch(1.0).into(),
                            right: Units::Stretch(1.0).into(),
                            font_size: StyleProp::Value(font_size),
                            ..Default::default()
                        }}
                        text={TextProps {
                            alignment: Alignment::Start,
                            content: button.text.clone(),
                            ..Default::default()
                        }}
                    />
                </ElementBundle>
            };
        }
    }

    true
}
