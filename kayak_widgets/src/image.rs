use kayak_ui::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    widget, Children,
};

#[widget]
pub fn Image(handle: u16, children: Children) {
    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Image { handle }),
        ..styles.clone().unwrap_or_default()
    });

    rsx! {
        <>
            {children}
        </>
    }
}
