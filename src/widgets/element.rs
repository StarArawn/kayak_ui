use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    widget, Children,
};

#[widget]
pub fn Element(children: Children) {
    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Layout),
        ..styles.clone().unwrap_or_default()
    });

    rsx! {
        <>
            {children}
        </>
    }
}
