use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp, Units},
    widget, Children,
};

#[widget]
pub fn Clip(children: Children, styles: Option<Style>) {
    let incoming_styles = styles.clone().unwrap_or_default();
    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Clip),
        width: if matches!(incoming_styles.width, StyleProp::Value(..)) { incoming_styles.width } else { StyleProp::Value(Units::Stretch(1.0)) },
        height: if matches!(incoming_styles.height, StyleProp::Value(..)) { incoming_styles.height } else { StyleProp::Value(Units::Stretch(1.0)) },
        // min_width: StyleProp::Value(Units::Stretch(1.0)),
        // min_height: StyleProp::Value(Units::Stretch(1.0)),
        ..incoming_styles
    });
    rsx! {
        <>
            {children}
        </>
    }
}
