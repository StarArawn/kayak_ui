use kayak_core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    widget, Children, Fragment,
};

#[widget]
pub fn Background(children: Children, styles: Option<Style>) {
    if styles.is_none() {
        *styles = Some(Style::default())
    }
    styles.as_mut().unwrap().render_command = StyleProp::Value(RenderCommand::Quad);
    rsx! {
        <Fragment>
            {children}
        </Fragment>
    }
}
