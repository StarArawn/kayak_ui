use kayak_ui::{
    core::{
        styles::{LayoutType, Style, StyleProp, Units},
        Bound, constructor, EventType, Handler, KeyCode, OnEvent,
        rsx, use_state, VecTracker, widget,
    },
    widgets::{Background, Element, Text},
};

use crate::tab::Tab;
use crate::TabTheme;

#[widget]
pub fn TabBar(context: &mut KayakContext, tabs: Vec<String>, selected: usize, on_select_tab: Handler<usize>, on_add_tab: Handler, on_remove_tab: Handler<usize>) {
    let theme = context.create_consumer::<TabTheme>().unwrap_or_default();
    let (is_add_hovered, set_is_add_hovered, ..) = use_state!(false);

    let tabs = tabs.iter().enumerate().map(|(index, tab)| {
        let on_select = on_select_tab.clone();
        let tab_event_handler = OnEvent::new(move |_, event| {
            match event.event_type {
                EventType::Click => {
                    on_select.call(index);
                }
                EventType::KeyDown(evt) => {
                    if evt.key() == KeyCode::Return || evt.key() == KeyCode::Space {
                        on_select.call(index);
                    }
                }
                _ => {}
            }
        });

        let on_remove = on_remove_tab.clone();
        let on_request_remove = Handler::new(move |_| {
            on_remove.call(index);
        });

        constructor! {
            <Tab content={tab.clone()} on_event={Some(tab_event_handler.clone())} selected={selected == index} on_request_remove={on_request_remove} />
        }
    }).collect::<Vec<_>>();

    let add_btn_event_handler = OnEvent::new(move |_, event| match event.event_type {
        EventType::Hover => {
            set_is_add_hovered(true);
        }
        EventType::MouseOut => {
            set_is_add_hovered(false);
        }
        EventType::Focus => {
            set_is_add_hovered(true);
        }
        EventType::Blur => {
            set_is_add_hovered(false);
        }
        EventType::Click => {
            on_add_tab.call(());
        }
        EventType::KeyDown(evt) => {
            if evt.key() == KeyCode::Return || evt.key() == KeyCode::Space {
                on_add_tab.call(());
            }
        }
        _ => {}
    });

    let add_btn_styles = Style {
        height: StyleProp::Value(Units::Pixels(theme.get().tab_height)),
        width: StyleProp::Value(Units::Pixels(theme.get().tab_height)),
        border_radius: StyleProp::Value((10.0, 10.0, 10.0, 10.0)),
        ..Default::default()
    };
    let add_btn_text_styles = Style {
        color: if is_add_hovered {
            StyleProp::Value(theme.get().text.hovered)
        } else {
            StyleProp::Value(theme.get().text.normal)
        },
        left: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Percentage(100.0)),
        width: StyleProp::Value(Units::Percentage(100.0)),
        ..Default::default()
    };

    let background_styles = Style {
        layout_type: StyleProp::Value(LayoutType::Row),
        background_color: StyleProp::Value(theme.get().bg),
        height: StyleProp::Value(Units::Auto),
        width: StyleProp::Value(Units::Stretch(1.0)),
        ..styles.clone().unwrap_or_default()
    };

    rsx! {
        <Background styles={Some(background_styles)}>
            <VecTracker data={tabs} />
            <Element focusable={Some(true)} on_event={Some(add_btn_event_handler)} styles={Some(add_btn_styles)}>
                <Text content={"+".to_string()} size={16.0} styles={Some(add_btn_text_styles)} />
            </Element>
        </Background>
    }
}