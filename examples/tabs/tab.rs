use kayak_ui::{
    core::{
        render_command::RenderCommand,
        styles::{LayoutType, Style, StyleProp, Units},
        Bound, EventType, Handler, KeyCode, OnEvent, rsx, use_state, widget,
    },
    widgets::{Background, Text},
};

use crate::TabTheme;

#[derive(Clone, PartialEq)]
enum TabHoverState {
    None,
    Inactive,
    Active,
}

#[widget]
pub fn Tab(context: &mut KayakContext, content: String, selected: bool, on_request_remove: Handler) {
    let theme = context.create_consumer::<TabTheme>().unwrap_or_default();
    let (hover_state, set_hover_state, ..) = use_state!(TabHoverState::None);
    match hover_state {
        TabHoverState::Inactive if selected => set_hover_state(TabHoverState::Active),
        TabHoverState::Active if !selected => set_hover_state(TabHoverState::Inactive),
        _ => {}
    };
    let (focus_state, set_focus_state, ..) = use_state!(false);
    let (is_exit_hovered, set_is_exit_hovered, ..) = use_state!(false);

    let event_handler = OnEvent::new(move |_, event| {
        match event.event_type {
            EventType::Hover => {
                if selected {
                    set_hover_state(TabHoverState::Active);
                } else {
                    set_hover_state(TabHoverState::Inactive);
                }
            }
            EventType::MouseOut => {
                set_hover_state(TabHoverState::None);
            }
            EventType::Focus => {
                set_focus_state(true);
            }
            EventType::Blur => {
                set_focus_state(false);
            }
            _ => {}
        }
    });

    let exit_btn_event_handler = OnEvent::new(move |_, event| {
        match event.event_type {
            EventType::Hover => {
                set_is_exit_hovered(true);
            }
            EventType::MouseOut => {
                set_is_exit_hovered(false);
            }
            EventType::Focus => {
                set_is_exit_hovered(true);
            }
            EventType::Blur => {
                set_is_exit_hovered(false);
            }
            EventType::Click => {
                // Stop propagation so we don't select a deleted tab!
                event.stop_propagation();
                on_request_remove.call(());
            }
            EventType::KeyDown(evt) => {
                if evt.key() == KeyCode::Return || evt.key() == KeyCode::Space {
                    // Stop propagation so we don't select a deleted tab!
                    event.stop_propagation();
                    on_request_remove.call(());
                }
            }
            _ => {}
        }
    });

    let tab_color = match hover_state {
        TabHoverState::None if selected => theme.get().active_tab.normal,
        TabHoverState::None => theme.get().inactive_tab.normal,
        TabHoverState::Inactive => theme.get().inactive_tab.hovered,
        TabHoverState::Active => theme.get().active_tab.hovered,
    };

    let pad_x = Units::Pixels(2.0);
    let bg_styles = Style {
        background_color: StyleProp::Value(tab_color),
        layout_type: StyleProp::Value(LayoutType::Row),
        padding_left: StyleProp::Value(pad_x),
        padding_right: StyleProp::Value(pad_x),
        ..Default::default()
    };


    let border_width = Units::Pixels(2.0);
    let border_styles = Style {
        background_color: if focus_state {
            StyleProp::Value(theme.get().focus)
        } else {
            StyleProp::Value(tab_color)
        },
        padding_left: StyleProp::Value(border_width),
        padding_right: StyleProp::Value(border_width),
        padding_top: StyleProp::Value(border_width),
        padding_bottom: StyleProp::Value(border_width),
        layout_type: StyleProp::Value(LayoutType::Row),
        ..Default::default()
    };

    let text_styles = Style {
        background_color: if focus_state {
            StyleProp::Value(theme.get().focus)
        } else {
            StyleProp::Value(tab_color)
        },
        color: StyleProp::Value(theme.get().text.normal),
        top: StyleProp::Value(Units::Stretch(0.1)),
        bottom: StyleProp::Value(Units::Stretch(1.0)),
        width: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };

    let exit_styles = Style {
        background_color: if is_exit_hovered {
            let mut darkened = theme.get().inactive_tab.hovered;
            darkened.r -= 0.025;
            darkened.g -= 0.025;
            darkened.b -= 0.025;
            StyleProp::Value(darkened)
        } else {
            StyleProp::Value(tab_color)
        },
        width: StyleProp::Value(Units::Pixels(theme.get().tab_height - 4.0)),
        height: StyleProp::Value(Units::Pixels(theme.get().tab_height - 4.0)),
        top: StyleProp::Value(Units::Stretch(0.175)),
        bottom: StyleProp::Value(Units::Stretch(1.0)),
        left: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };

    let exit_text_styles = Style {
        left: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        top: StyleProp::Value(Units::Stretch(0.35)),
        bottom: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };

    self.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Layout),
        height: StyleProp::Value(Units::Pixels(theme.get().tab_height)),
        max_width: StyleProp::Value(Units::Pixels(100.0)),
        ..styles.clone().unwrap_or_default()
    });

    rsx! {
        <Background focusable={Some(true)} on_event={Some(event_handler)} styles={Some(border_styles)}>
            <Background styles={Some(bg_styles)}>
                <Text content={content} size={12.0} styles={Some(text_styles)} />
                <Background focusable={Some(true)} on_event={Some(exit_btn_event_handler)} styles={Some(exit_styles)}>
                    <Text content={"X".to_string()} size={8.0} styles={Some(exit_text_styles)} />
                </Background>
            </Background>
        </Background>
    }
}