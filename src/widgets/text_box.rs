use crate::core::{
    render_command::RenderCommand,
    Children, rsx, WidgetProps,
    styles::{Style, StyleProp, Units},
    widget, Bound, Color, EventType, MutableBound, OnEvent,
};
use std::sync::{Arc, RwLock};

use crate::widgets::{Background, Clip, Text};

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct TextBoxProps {
    pub value: String,
    pub on_change: Option<OnChange>,
    pub placeholder: Option<String>,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(Children)]

    pub children: Option<Children>,
    #[prop_field(OnEvent)]

    pub on_event: Option<OnEvent>,
    #[prop_field(Focusable)]

    pub focusable: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChangeEvent {
    pub value: String,
}

#[derive(Clone)]
pub struct OnChange(pub Arc<RwLock<dyn FnMut(ChangeEvent) + Send + Sync + 'static>>);

impl OnChange {
    pub fn new<F: FnMut(ChangeEvent) + Send + Sync + 'static>(f: F) -> OnChange {
        OnChange(Arc::new(RwLock::new(f)))
    }
}

impl PartialEq for OnChange {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl std::fmt::Debug for OnChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OnChange").finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Focus(pub bool);

#[widget(focusable)]
pub fn TextBox(props: TextBoxProps) {
    let TextBoxProps {on_change, placeholder, value, ..} = props.clone();
    let current_styles = props.styles.clone().unwrap_or_default();
    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Layout),
        height: StyleProp::Value(Units::Pixels(26.0)),
        top: if matches!(current_styles.top, StyleProp::Value { .. }) {
            current_styles.top.clone()
        } else {
            StyleProp::Value(Units::Pixels(0.0))
        },
        bottom: if matches!(current_styles.bottom, StyleProp::Value { .. }) {
            current_styles.top.clone()
        } else {
            StyleProp::Value(Units::Pixels(0.0))
        },
        ..current_styles
    });

    let background_styles = Style {
        background_color: StyleProp::Value(Color::new(0.176, 0.196, 0.215, 1.0)),
        border_radius: StyleProp::Value((5.0, 5.0, 5.0, 5.0)),
        height: StyleProp::Value(Units::Pixels(26.0)),
        padding_left: StyleProp::Value(Units::Pixels(5.0)),
        padding_right: StyleProp::Value(Units::Pixels(5.0)),
        ..props.styles.clone().unwrap_or_default()
    };

    let has_focus = context.create_state(Focus(false)).unwrap();

    let mut current_value = value.clone();
    let cloned_on_change = on_change.clone();
    let cloned_has_focus = has_focus.clone();

    props.on_event = Some(OnEvent::new(move |_, event| match event.event_type {
        EventType::CharInput { c } => {
            if !cloned_has_focus.get().0 {
                return;
            }
            if is_backspace(c) {
                if !current_value.is_empty() {
                    current_value.truncate(current_value.len() - 1);
                }
            } else if !c.is_control() {
                current_value.push(c);
            }
            if let Some(on_change) = cloned_on_change.as_ref() {
                if let Ok(mut on_change) = on_change.0.write() {
                    on_change(ChangeEvent {
                        value: current_value.clone(),
                    });
                }
            }
        }
        EventType::Focus => cloned_has_focus.set(Focus(true)),
        EventType::Blur => cloned_has_focus.set(Focus(false)),
        _ => {}
    }));

    let text_styles = if value.is_empty() || (has_focus.get().0 && value.is_empty()) {
        Style {
            color: StyleProp::Value(Color::new(0.5, 0.5, 0.5, 1.0)),
            ..Style::default()
        }
    } else {
        Style {
            color: props.styles.clone().unwrap_or_default().color,
            ..Style::default()
        }
    };

    let value = if value.is_empty() {
        placeholder.unwrap_or_else(|| value.clone())
    } else {
        value
    };

    rsx! {
        <Background styles={Some(background_styles)}>
            <Clip>
                <Text
                    content={value}
                    size={14.0}
                    line_height={Some(22.0)}
                    styles={Some(text_styles)}
                />
            </Clip>
        </Background>
    }
}

/// Checks if the given character contains the "Backspace" sequence
///
/// Context: [Wikipedia](https://en.wikipedia.org/wiki/Backspace#Common_use)
fn is_backspace(c: char) -> bool {
    c == '\u{8}' || c == '\u{7f}'
}
