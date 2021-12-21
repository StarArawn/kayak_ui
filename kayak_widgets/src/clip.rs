use kayak_ui::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp, Units},
    widget, Children,
};

#[widget]
pub fn Clip(children: Children, styles: Option<Style>) {
    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Clip),
        width: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Stretch(1.0)),
        min_width: StyleProp::Value(Units::Stretch(1.0)),
        min_height: StyleProp::Value(Units::Stretch(1.0)),
        ..styles.clone().unwrap_or_default()
    });
    rsx! {
        <>
            {children}
        </>
    }
}
