use kayak_ui::core::{
    layout_cache::Space,
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    widget, Children,
};

#[widget]
pub fn NinePatch(handle: u16, border: Space, children: Children) {
    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::NinePatch {
            handle: *handle,
            border: *border,
        }),
        ..styles.clone().unwrap_or_default()
    });

    rsx! {
        <>
            {children}
        </>
    }
}
