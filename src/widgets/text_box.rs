use instant::Instant;

use bevy::prelude::*;
use kayak_font::{KayakFont, TextProperties};
use kayak_ui_macros::{constructor, rsx};

use crate::{
    context::WidgetName,
    event::{Event, EventType},
    event_dispatcher::EventDispatcherContext,
    on_event::OnEvent,
    on_layout::OnLayout,
    prelude::{KChildren, KayakWidgetContext, OnChange},
    render::font::FontMapping,
    styles::{ComputedStyles, Edge, KPositionType, KStyle, RenderCommand, StyleProp, Units},
    widget::Widget,
    widget_state::WidgetState,
    widgets::{
        text::{TextProps, TextWidgetBundle},
        BackgroundBundle, ClipBundle,
    },
    Focusable, DEFAULT_FONT,
};

use super::ElementBundle;

/// Props used by the [`TextBox`] widget
#[derive(Component, PartialEq, Eq, Default, Debug, Clone)]
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

#[derive(Component, Clone, PartialEq)]
pub struct TextBoxState {
    pub focused: bool,
    pub graphemes: Vec<String>,
    pub cursor_x: f32,
    pub cursor_position: usize,
    pub cursor_visible: bool,
    pub cursor_last_update: Instant,
    pub current_value: String,
}

impl Default for TextBoxState {
    fn default() -> Self {
        Self {
            focused: Default::default(),
            graphemes: Default::default(),
            cursor_x: 0.0,
            cursor_position: Default::default(),
            cursor_visible: Default::default(),
            cursor_last_update: Instant::now(),
            current_value: String::new(),
        }
    }
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
    pub computed_styles: ComputedStyles,
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
            computed_styles: ComputedStyles::default(),
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
    mut query: Query<(
        &KStyle,
        &mut ComputedStyles,
        &TextBoxProps,
        &mut OnEvent,
        &OnChange,
    )>,
    mut state_query: ParamSet<(Query<&TextBoxState>, Query<&mut TextBoxState>)>,
    font_assets: Res<Assets<KayakFont>>,
    font_mapping: Res<FontMapping>,
) -> bool {
    if let Ok((styles, mut computed_styles, text_box, mut on_event, on_change)) =
        query.get_mut(entity)
    {
        let state_entity = widget_context.use_state::<TextBoxState>(
            &mut commands,
            entity,
            TextBoxState {
                current_value: text_box.value.clone(),
                ..TextBoxState::default()
            },
        );

        let mut is_different = false;
        if let Ok(state) = state_query.p0().get(state_entity) {
            if state.current_value != text_box.value {
                is_different = true;
            }
        }

        let style_font = styles.font.clone();

        if is_different {
            if let Ok(mut state) = state_query.p1().get_mut(state_entity) {
                state.current_value = text_box.value.clone();
                // Update graphemes
                set_graphemes(&mut state, &font_assets, &font_mapping, &style_font);

                state.cursor_position = state.graphemes.len();

                set_new_cursor_position(&mut state, &font_assets, &font_mapping, &style_font);
            }
        }

        if let Ok(state) = state_query.p0().get(state_entity) {
            *computed_styles = KStyle::default()
                // Required styles
                .with_style(KStyle {
                    render_command: RenderCommand::Layout.into(),
                    ..Default::default()
                })
                // Apply any prop-given styles
                .with_style(styles)
                // If not set by props, apply these styles
                .with_style(KStyle {
                    top: Units::Pixels(0.0).into(),
                    bottom: Units::Pixels(0.0).into(),
                    height: Units::Pixels(26.0).into(),
                    // cursor: CursorIcon::Text.into(),
                    ..Default::default()
                })
                .into();

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

            let cloned_on_change = on_change.clone();

            *on_event = OnEvent::new(
                move |In((event_dispatcher_context, _, mut event, _entity)): In<(
                    EventDispatcherContext,
                    WidgetState,
                    Event,
                    Entity,
                )>,
                      font_assets: Res<Assets<KayakFont>>,
                      font_mapping: Res<FontMapping>,
                      mut state_query: Query<&mut TextBoxState>| {
                    match event.event_type {
                        EventType::KeyDown(key_event) => {
                            if key_event.key() == KeyCode::Right {
                                if let Ok(mut state) = state_query.get_mut(state_entity) {
                                    if state.cursor_position < state.graphemes.len() {
                                        state.cursor_position += 1;
                                    }
                                    set_new_cursor_position(
                                        &mut state,
                                        &font_assets,
                                        &font_mapping,
                                        &style_font,
                                    );
                                }
                            }
                            if key_event.key() == KeyCode::Left {
                                if let Ok(mut state) = state_query.get_mut(state_entity) {
                                    if state.cursor_position > 0 {
                                        state.cursor_position -= 1;
                                    }
                                    set_new_cursor_position(
                                        &mut state,
                                        &font_assets,
                                        &font_mapping,
                                        &style_font,
                                    );
                                }
                            }
                        }
                        EventType::CharInput { c } => {
                            if let Ok(mut state) = state_query.get_mut(state_entity) {
                                let cloned_on_change = cloned_on_change.clone();
                                if !state.focused {
                                    return (event_dispatcher_context, event);
                                }
                                let cursor_pos = state.cursor_position;
                                if is_backspace(c) {
                                    if !state.current_value.is_empty() {
                                        let char_pos: usize = state.graphemes[0..cursor_pos - 1]
                                            .iter()
                                            .map(|g| g.len())
                                            .sum();
                                        state.current_value.remove(char_pos);
                                        state.cursor_position -= 1;
                                    }
                                } else if !c.is_control() {
                                    let char_pos: usize = state.graphemes[0..cursor_pos]
                                        .iter()
                                        .map(|g| g.len())
                                        .sum();
                                    state.current_value.insert(char_pos, c);

                                    state.cursor_position += 1;
                                }

                                // Update graphemes
                                set_graphemes(&mut state, &font_assets, &font_mapping, &style_font);

                                set_new_cursor_position(
                                    &mut state,
                                    &font_assets,
                                    &font_mapping,
                                    &style_font,
                                );
                                cloned_on_change.set_value(state.current_value.clone());
                                event.add_system(cloned_on_change);
                            }
                        }
                        EventType::Focus => {
                            if let Ok(mut state) = state_query.get_mut(state_entity) {
                                state.focused = true;
                                // Update graphemes
                                set_graphemes(&mut state, &font_assets, &font_mapping, &style_font);

                                state.cursor_position = state.graphemes.len();

                                set_new_cursor_position(
                                    &mut state,
                                    &font_assets,
                                    &font_mapping,
                                    &style_font,
                                );
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

            let cursor_styles = KStyle {
                background_color: Color::rgba(0.933, 0.745, 0.745, 1.0).into(),
                position_type: KPositionType::SelfDirected.into(),
                top: Units::Pixels(5.0).into(),
                left: Units::Pixels(state.cursor_x).into(),
                width: Units::Pixels(2.0).into(),
                height: Units::Pixels(26.0 - 10.0).into(),
                ..Default::default()
            };

            let text_styles = KStyle {
                top: Units::Stretch(1.0).into(),
                bottom: Units::Stretch(1.0).into(),
                ..Default::default()
            };

            let shift = if let Some(layout) = widget_context.get_layout(entity) {
                let font_handle = match &styles.font {
                    StyleProp::Value(font) => font_mapping.get_handle(font.clone()).unwrap(),
                    _ => font_mapping.get_handle(DEFAULT_FONT.into()).unwrap(),
                };
                if let Some(font) = font_assets.get(&font_handle) {
                    let string_to_cursor = state.graphemes[0..state.cursor_position].join("");
                    let measurement = font.measure(
                        &string_to_cursor,
                        TextProperties {
                            font_size: 14.0,
                            line_height: 18.0,
                            max_size: (10000.0, 18.0),
                            alignment: kayak_font::Alignment::Start,
                            tab_size: 4,
                        },
                    );
                    if measurement.size().0 > layout.width {
                        (layout.width - measurement.size().0) - 20.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let scroll_styles = KStyle {
                position_type: KPositionType::SelfDirected.into(),
                padding_left: StyleProp::Value(Units::Stretch(0.0)),
                padding_right: StyleProp::Value(Units::Stretch(0.0)),
                padding_bottom: StyleProp::Value(Units::Stretch(1.0)),
                padding_top: StyleProp::Value(Units::Stretch(1.0)),
                left: Units::Pixels(shift).into(),
                ..Default::default()
            };

            let parent_id = Some(entity);
            rsx! {
                <BackgroundBundle styles={background_styles}>
                    <ClipBundle styles={KStyle {
                        height: Units::Pixels(26.0).into(),
                        padding_left: StyleProp::Value(Units::Stretch(0.0)),
                        padding_right: StyleProp::Value(Units::Stretch(0.0)),
                        ..Default::default()
                    }}>
                        <ElementBundle styles={scroll_styles}>
                            <TextWidgetBundle
                                styles={text_styles}
                                text={TextProps {
                                    content: text_box.value.clone(),
                                    size: 14.0,
                                    line_height: Some(18.0),
                                    word_wrap: false,
                                    ..Default::default()
                                }}
                            />
                            {
                                if state.focused && state.cursor_visible {
                                    constructor! {
                                        <BackgroundBundle styles={cursor_styles} />
                                    }
                                }
                            }
                        </ElementBundle>
                    </ClipBundle>
                </BackgroundBundle>
            };
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

fn set_graphemes(
    state: &mut TextBoxState,
    font_assets: &Res<Assets<KayakFont>>,
    font_mapping: &FontMapping,
    style_font: &StyleProp<String>,
) {
    let font_handle = match style_font {
        StyleProp::Value(font) => font_mapping.get_handle(font.clone()).unwrap(),
        _ => font_mapping.get_handle(DEFAULT_FONT.into()).unwrap(),
    };

    if let Some(font) = font_assets.get(&font_handle) {
        state.graphemes = font
            .get_graphemes(&state.current_value)
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
    }
}

fn get_single_grapheme_length(
    font_assets: &Res<Assets<KayakFont>>,
    font_mapping: &FontMapping,
    style_font: &StyleProp<String>,
    text: &String,
) -> usize {
    let font_handle = match style_font {
        StyleProp::Value(font) => font_mapping.get_handle(font.clone()).unwrap(),
        _ => font_mapping.get_handle(DEFAULT_FONT.into()).unwrap(),
    };

    if let Some(font) = font_assets.get(&font_handle) {
        let graphemes = font.get_graphemes(&text);
        return graphemes[0].len();
    }

    0
}

fn set_new_cursor_position(
    state: &mut TextBoxState,
    font_assets: &Res<Assets<KayakFont>>,
    font_mapping: &FontMapping,
    style_font: &StyleProp<String>,
) {
    let font_handle = match style_font {
        StyleProp::Value(font) => font_mapping.get_handle(font.clone()).unwrap(),
        _ => font_mapping.get_handle(DEFAULT_FONT.into()).unwrap(),
    };

    if let Some(font) = font_assets.get(&font_handle) {
        let string_to_cursor = state.graphemes[0..state.cursor_position].join("");
        let measurement = font.measure(
            &string_to_cursor,
            TextProperties {
                font_size: 14.0,
                line_height: 18.0,
                max_size: (10000.0, 18.0),
                alignment: kayak_font::Alignment::Start,
                tab_size: 4,
            },
        );

        state.cursor_x = measurement.size().0;
    }
}

pub fn cursor_animation_system(
    mut state_query: ParamSet<(Query<(Entity, &TextBoxState)>, Query<&mut TextBoxState>)>,
) {
    let mut should_update = Vec::new();

    for (entity, state) in state_query.p0().iter() {
        if state.cursor_last_update.elapsed().as_secs_f32() > 0.5 && state.focused {
            should_update.push(entity);
        }
    }

    for state_entity in should_update.drain(..) {
        if let Ok(mut state) = state_query.p1().get_mut(state_entity) {
            state.cursor_last_update = Instant::now();
            state.cursor_visible = !state.cursor_visible;
        }
    }
}
