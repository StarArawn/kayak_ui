use kayak_ui::core::{
    render_command::RenderCommand,
    styles::{Style, StyleProp},
    widget,
};

#[widget]
pub fn Text(size: f32, content: String, styles: Option<Style>) {
    let render_command = RenderCommand::Text {
        content,
        size,
        font: 0, // TODO: Support font passing here. Perhaps move to style?
    };
    *styles = Some(Style {
        render_command: StyleProp::Value(render_command),
        ..styles.clone().unwrap_or_default()
    });
    // rsx! {
    //     <>
    //         {}
    //     </>
    // }
}
