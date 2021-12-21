use kayak_ui::core::{
    render_command::RenderCommand,
    styles::{Style, StyleProp},
    widget,
};

#[widget]
pub fn Text(size: f32, content: String, styles: Option<Style>, font: Option<u16>) {
    let render_command = RenderCommand::Text {
        content,
        size,
        font: font.unwrap_or(0),
    };
    *styles = Some(Style {
        render_command: StyleProp::Value(render_command),
        ..styles.clone().unwrap_or_default()
    });
}
