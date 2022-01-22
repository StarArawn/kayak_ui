use kayak_ui::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    use_state, widget, Bound, Fragment, Handler,
};
use std::fmt::Debug;

use crate::tab_bar::TabBar;
use crate::tab_content::TabContent;
use crate::TabTheme;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TabData {
    /// The name of this tab
    pub name: String,
    /// The content to display for this tab, wrapped in a [Fragment]
    pub content: Fragment,
}

/// The actual tab container widget.
///
/// This houses both the tab bar and its content.
#[widget]
pub fn TabBox(context: &mut KayakContext, tabs: Vec<TabData>, initial_tab: usize) {
    let theme = context.create_consumer::<TabTheme>().unwrap_or_default();
    let (selected, set_selected, ..) = use_state!(initial_tab);

    let tab_names = tabs
        .iter()
        .map(|tab| tab.name.clone())
        .collect::<Vec<String>>();
    let tab_content = tabs
        .iter()
        .map(|tab| tab.content.clone())
        .collect::<Vec<_>>();

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
            <TabBar tabs={tab_names} selected={selected} on_select_tab={on_select_tab} />
            <TabContent tabs={tab_content} selected={selected}  />
        </>
    }
}
