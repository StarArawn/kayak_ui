use std::fmt::Debug;
use kayak_ui::{
    core::{
        render_command::RenderCommand,
        styles::{Style, StyleProp},
        Bound, Fragment, Handler, rsx, use_state, widget,
    }
};

use crate::tab_bar::TabBar;
use crate::TabTheme;
use crate::tab_content::TabContent;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TabData {
    pub name: String,
    pub content: Fragment,
}

#[widget]
pub fn TabBox(context: &mut KayakContext, tabs: Vec<TabData>, initial_tab: usize, on_add_tab: Handler, on_remove_tab: Handler<usize>) {
    let theme = context.create_consumer::<TabTheme>().unwrap_or_default();
    let (selected, set_selected, ..) = use_state!(initial_tab);

    let tab_names = tabs.iter().map(|tab| {
        tab.name.clone()
    }).collect::<Vec<String>>();
    let tab_content = tabs.iter().map(|tab| {
        tab.content.clone()
    }).collect::<Vec<_>>();

    let on_select_tab = Handler::<usize>::new(move |index| {
        set_selected(index);
    });

    self.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Quad),
        background_color: StyleProp::Value(theme.get().fg),
        ..Default::default()
    });

    rsx! {
        <>
            <TabBar tabs={tab_names} selected={selected} on_select_tab={on_select_tab} on_add_tab={on_add_tab} on_remove_tab={on_remove_tab} />
            <TabContent tabs={tab_content} selected={selected}  />
        </>
    }
}