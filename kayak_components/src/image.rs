use kayak_core::{component, rsx, Render, Update};

#[component]
pub fn Image<Children: Render + Update + Clone>(handle: u16, children: Children) {
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
