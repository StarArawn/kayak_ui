use kayak_ui::{
    core::{
        render_command::RenderCommand,
        rsx, WidgetProps,
        styles::{LayoutType, Style, StyleProp, Units},
        use_state, widget, Bound, EventType, OnEvent,
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

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct TabProps {
    pub content: String,
    pub selected: bool,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(OnEvent)]
    pub on_event: Option<OnEvent>,
}

/// The actual tab, displayed in a [TabBar](crate::tab_bar::TabBar)
#[widget]
pub fn Tab(props: TabProps) {
    let TabProps{content, selected, ..} = props.clone();

    let theme = context.create_consumer::<TabTheme>().unwrap_or_default();
    let (focus_state, set_focus_state, ..) = use_state!(false);
    let (hover_state, set_hover_state, ..) = use_state!(TabHoverState::None);
    match hover_state {
        TabHoverState::Inactive if selected => set_hover_state(TabHoverState::Active),
        TabHoverState::Active if !selected => set_hover_state(TabHoverState::Inactive),
        _ => {}
    };

    let event_handler = OnEvent::new(move |_, event| match event.event_type {
        EventType::Hover(..) => {
            if selected {
                set_hover_state(TabHoverState::Active);
            } else {
                set_hover_state(TabHoverState::Inactive);
            }
        }
        EventType::MouseOut(..) => {
            set_hover_state(TabHoverState::None);
        }
        EventType::Focus => {
            set_focus_state(true);
        }
        EventType::Blur => {
            set_focus_state(false);
        }
        _ => {}
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

    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Layout),
        height: StyleProp::Value(Units::Pixels(theme.get().tab_height)),
        max_width: StyleProp::Value(Units::Pixels(100.0)),
        ..props.styles.clone().unwrap_or_default()
    });

    rsx! {
        <Background focusable={Some(true)} on_event={Some(event_handler)} styles={Some(border_styles)}>
            <Background styles={Some(bg_styles)}>
                <Text content={content} size={12.0} styles={Some(text_styles)} />
            </Background>
        </Background>
    }
}
