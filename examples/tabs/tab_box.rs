use kayak_ui::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    use_state, widget, Bound, Fragment, Handler, WidgetProps,
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

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct TabBoxProps {
    pub initial_tab: usize,
    pub tabs: Vec<TabData>,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
}

/// The actual tab container widget.
///
/// This houses both the tab bar and its content.
#[widget]
pub fn TabBox(props: TabBoxProps) {
    let TabBoxProps {
        initial_tab, tabs, ..
    } = props.clone();
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

    props.styles = Some(Style {
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
