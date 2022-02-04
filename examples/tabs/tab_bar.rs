use kayak_ui::{
    core::{
        constructor, rsx,
        styles::{LayoutType, Style, StyleProp, Units},
        widget, Bound, EventType, Handler, KeyCode, OnEvent, VecTracker, WidgetProps,
    },
    widgets::Background,
};

use crate::tab::Tab;
use crate::TabTheme;

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct TabBarProps {
    pub tabs: Vec<String>,
    pub selected: usize,
    pub on_select_tab: Handler<usize>,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
}

/// A widget displaying a collection of tabs in a horizontal bar
#[widget]
pub fn TabBar(props: TabBarProps) {
    let TabBarProps {
        on_select_tab,
        selected,
        tabs,
        ..
    } = props.clone();
    let theme = context.create_consumer::<TabTheme>().unwrap_or_default();

    let tabs = tabs.into_iter().enumerate().map(move |(index, tab)| {
        let on_select = on_select_tab.clone();
        let tab_event_handler = OnEvent::new(move |_, event| {
            match event.event_type {
                EventType::Click(..) =>  {
                    on_select.call(index);
                }
                EventType::KeyDown(evt) => {
                    if evt.key() == KeyCode::Return || evt.key() == KeyCode::Space {
                        // We want the focused tab to also be selected by `Enter` or `Space`
                        on_select.call(index);
                    }
                }
                _ => {}
            }
        });

        constructor! {
            <Tab content={tab.clone()} on_event={Some(tab_event_handler.clone())} selected={selected == index} />
        }
    });

    let background_styles = Style {
        layout_type: StyleProp::Value(LayoutType::Row),
        background_color: StyleProp::Value(theme.get().bg),
        height: StyleProp::Value(Units::Auto),
        width: StyleProp::Value(Units::Stretch(1.0)),
        ..props.styles.clone().unwrap_or_default()
    };

    rsx! {
        <Background styles={Some(background_styles)}>
            {VecTracker::from(tabs.clone())}
        </Background>
    }
}
