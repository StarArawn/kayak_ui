use bevy::prelude::{Bundle, Color, Commands, Component, Entity, In, Query};
use kayak_ui_macros::rsx;

use crate::{
    context::WidgetName,
    event::{Event, EventType},
    event_dispatcher::EventDispatcherContext,
    on_event::OnEvent,
    on_layout::OnLayout,
    prelude::{KChildren, KayakWidgetContext, OnChange},
    styles::{Edge, KStyle, RenderCommand, StyleProp, Units},
    widget::Widget,
    widget_state::WidgetState,
    widgets::{
        text::{TextProps, TextWidgetBundle},
        BackgroundBundle, ClipBundle,
    },
    Focusable,
};

/// Props used by the [`TextBox`] widget
#[derive(Component, PartialEq, Default, Debug, Clone)]
pub struct TextBoxProps {
    /// If true, prevents the widget from being focused (and consequently edited)
    pub disabled: bool,
    /// The text to display when the user input is empty
    pub placeholder: Option<String>,
    /// The user input
    ///
    /// This is a controlled state. You _must_ set this to the value to you wish to be displayed.
    /// You can use the [`on_change`] callback to update this prop as the user types.
    pub value: String,
}

#[derive(Component, Default, Clone, PartialEq)]
pub struct TextBoxState {
    pub focused: bool,
}

pub struct TextBoxValue(pub String);

impl Widget for TextBoxProps {}

/// A widget that displays a text input field
/// A text box allows users to input text.
/// This text box is fairly simple and only supports basic input.
///
#[derive(Bundle)]
pub struct TextBoxBundle {
    pub text_box: TextBoxProps,
    pub styles: KStyle,
    pub on_event: OnEvent,
    pub on_layout: OnLayout,
    pub on_change: OnChange,
    pub focusable: Focusable,
    pub widget_name: WidgetName,
}

impl Default for TextBoxBundle {
    fn default() -> Self {
        Self {
            text_box: Default::default(),
            styles: Default::default(),
            on_event: Default::default(),
            on_layout: Default::default(),
            on_change: Default::default(),
            focusable: Default::default(),
            widget_name: TextBoxProps::default().get_name(),
        }
    }
}

pub fn text_box_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<(&mut KStyle, &TextBoxProps, &mut OnEvent, &OnChange)>,
    state_query: Query<&TextBoxState>,
) -> bool {
    if let Ok((mut styles, text_box, mut on_event, on_change)) = query.get_mut(entity) {
        let state_entity = widget_context.use_state::<TextBoxState>(
            &mut commands,
            entity,
            TextBoxState::default(),
        );

        if let Ok(state) = state_query.get(state_entity) {
            *styles = KStyle::default()
                // Required styles
                .with_style(KStyle {
                    render_command: RenderCommand::Layout.into(),
                    ..Default::default()
                })
                // Apply any prop-given styles
                .with_style(&*styles)
                // If not set by props, apply these styles
                .with_style(KStyle {
                    top: Units::Pixels(0.0).into(),
                    bottom: Units::Pixels(0.0).into(),
                    height: Units::Pixels(26.0).into(),
                    // cursor: CursorIcon::Text.into(),
                    ..Default::default()
                });

            let background_styles = KStyle {
                render_command: StyleProp::Value(RenderCommand::Quad),
                background_color: Color::rgba(0.160, 0.172, 0.235, 1.0).into(),
                border_color: if state.focused {
                    Color::rgba(0.933, 0.745, 0.745, 1.0).into()
                } else {
                    Color::rgba(0.360, 0.380, 0.474, 1.0).into()
                },
                border: Edge::new(0.0, 0.0, 0.0, 2.0).into(),
                height: Units::Pixels(26.0).into(),
                padding_left: Units::Pixels(5.0).into(),
                padding_right: Units::Pixels(5.0).into(),
                ..Default::default()
            };

            let current_value = text_box.value.clone();
            let cloned_on_change = on_change.clone();

            *on_event = OnEvent::new(
                move |In((event_dispatcher_context, _, mut event, _entity)): In<(
                    EventDispatcherContext,
                    WidgetState,
                    Event,
                    Entity,
                )>,
                      mut state_query: Query<&mut TextBoxState>| {
                    match event.event_type {
                        EventType::CharInput { c } => {
                            let mut current_value = current_value.clone();
                            let cloned_on_change = cloned_on_change.clone();
                            if let Ok(state) = state_query.get(state_entity) {
                                if !state.focused {
                                    return (event_dispatcher_context, event);
                                }
                            } else {
                                return (event_dispatcher_context, event);
                            }
                            if is_backspace(c) {
                                if !current_value.is_empty() {
                                    current_value.truncate(current_value.len() - 1);
                                }
                            } else if !c.is_control() {
                                current_value.push(c);
                            }
                            cloned_on_change.set_value(current_value);
                            event.add_system(cloned_on_change);
                        }
                        EventType::Focus => {
                            if let Ok(mut state) = state_query.get_mut(state_entity) {
                                state.focused = true;
                            }
                        }
                        EventType::Blur => {
                            if let Ok(mut state) = state_query.get_mut(state_entity) {
                                state.focused = false;
                            }
                        }
                        _ => {}
                    }
                    (event_dispatcher_context, event)
                },
            );

            let parent_id = Some(entity);
            rsx! {
                <BackgroundBundle styles={background_styles}>
                    <ClipBundle styles={KStyle {
                        height: Units::Pixels(26.0).into(),
                        padding_left: StyleProp::Value(Units::Stretch(0.0)),
                        padding_right: StyleProp::Value(Units::Stretch(0.0)),
                        padding_bottom: StyleProp::Value(Units::Stretch(1.0)),
                        padding_top: StyleProp::Value(Units::Stretch(1.0)),
                        ..Default::default()
                    }}>
                        <TextWidgetBundle
                            text={TextProps {
                                content: text_box.value.clone(),
                                size: 14.0,
                                line_height: Some(18.0),
                                user_styles: KStyle {
                                    top: Units::Stretch(1.0).into(),
                                    bottom: Units::Stretch(1.0).into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }}
                        />
                    </ClipBundle>
                </BackgroundBundle>
            }
        }
    }

    true
}

/// Checks if the given character contains the "Backspace" sequence
///
/// Context: [Wikipedia](https://en.wikipedia.org/wiki/Backspace#Common_use)
fn is_backspace(c: char) -> bool {
    c == '\u{8}' || c == '\u{7f}'
}
