use std::ops::Index;
use kayak_ui::{
    core::{
        render_command::RenderCommand,
        styles::{Style, StyleProp},
        Bound, Fragment, rsx, VecTracker, widget,
    }
};

use crate::TabTheme;

#[widget]
pub fn TabContent(context: &mut KayakContext, tabs: Vec<Fragment>, selected: usize) {
    let theme = context.create_consumer::<TabTheme>().unwrap_or_default();

    if selected >= tabs.len() {
        return;
    }

    self.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Quad),
        background_color: StyleProp::Value(theme.get().fg),
        ..Default::default()
    });

    let tab = tabs.index(selected).clone();

    rsx! {
        <>
            <VecTracker data={vec![tab.clone()]} />
        </>
    }
}