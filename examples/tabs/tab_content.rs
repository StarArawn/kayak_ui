use kayak_ui::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    widget, Bound, Fragment, VecTracker, WidgetProps,
};
use std::ops::Index;

use crate::TabTheme;

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct TabContentProps {
    pub selected: usize,
    pub tabs: Vec<Fragment>,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
}

/// A widget that displays the selected tab's content
#[widget]
pub fn TabContent(props: TabContentProps) {
    let TabContentProps { selected, tabs, .. } = props.clone();
    let theme = context.create_consumer::<TabTheme>().unwrap_or_default();

    if selected >= tabs.len() {
        // Invalid tab -> don't do anything
        return;
    }

    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Quad),
        background_color: StyleProp::Value(theme.get().fg),
        ..Default::default()
    });

    let tab = tabs.index(selected).clone();
    let tab = vec![tab.clone()];

    rsx! {
        <>
            {VecTracker::from(tab.clone().into_iter())}
        </>
    }
}
