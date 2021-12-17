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
pub fn TextBox(value: String, on_change: Option<OnChange>) {
    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Layout),
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

    let internal_value = context.create_state("".to_string()).unwrap();
    let has_focus = context.create_state(Focus(false)).unwrap();

    let cloned_on_change = on_change.clone();
    let cloned_has_focus = has_focus.clone();
    self.on_event = Some(OnEvent::new(move |_, event| match event.event_type {
        EventType::CharInput { c } => {
            if !cloned_has_focus.get().0 {
                return;
            }
            let mut current_value = internal_value.get();
            if c == '\u{8}' {
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
            internal_value.set(current_value);
        }
        EventType::Focus => {
            dbg!("Has focus!");
            cloned_has_focus.set(Focus(true))
        }
        EventType::Blur => {
            dbg!("Lost focus!");
            cloned_has_focus.set(Focus(false))
        }
        _ => {}
    }));

    // let cloned_has_focus = has_focus.clone();
    // let on_event = Some(OnEvent::new(move |_, event| match event.event_type {
    //     EventType::Focus => {
    //         dbg!("Has focus!");
    //         cloned_has_focus.set(Focus(true))
    //     }
    //     _ => {}
    // }));

    let value = value.clone();
    rsx! {
        <Background styles={Some(background_styles)}>
            <Clip>
                <Text content={value} size={14.0} />
            </Clip>
        </Background>
    }
}
