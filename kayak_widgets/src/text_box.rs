use kayak_ui::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp, Units},
    widget, Bound, Color, EventType, MutableBound, OnEvent,
};
use std::sync::{Arc, RwLock};

use crate::{Background, Clip, Text};

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
pub fn TextBox(value: String, on_change: Option<OnChange>, placeholder: Option<String>) {
    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Layout),
        height: StyleProp::Value(Units::Pixels(26.0)),
        top: StyleProp::Value(Units::Pixels(0.0)),
        bottom: StyleProp::Value(Units::Pixels(0.0)),
        ..styles.clone().unwrap_or_default()
    });

    let background_styles = Style {
        background_color: StyleProp::Value(Color::new(0.176, 0.196, 0.215, 1.0)),
        border_radius: StyleProp::Value((5.0, 5.0, 5.0, 5.0)),
        height: StyleProp::Value(Units::Pixels(26.0)),
        padding_left: StyleProp::Value(Units::Pixels(5.0)),
        padding_right: StyleProp::Value(Units::Pixels(5.0)),
        ..styles.clone().unwrap_or_default()
    };

    let has_focus = context.create_state(Focus(false)).unwrap();

    let mut current_value = value.clone();
    let cloned_on_change = on_change.clone();
    let cloned_has_focus = has_focus.clone();
    self.on_event = Some(OnEvent::new(move |_, event| match event.event_type {
        EventType::CharInput { c } => {
            if !cloned_has_focus.get().0 {
                return;
            }
            if is_backspace(c) {
                if current_value.len() > 0 {
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

    let value = if value.is_empty() {
        placeholder.clone().unwrap_or(value.clone())
    } else {
        value.clone()
    };
    rsx! {
        <Background styles={Some(background_styles)}>
            <Clip>
                <Text content={value} size={14.0} />
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