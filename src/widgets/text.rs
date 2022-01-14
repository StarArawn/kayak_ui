use kayak_core::{styles::Units, Binding, Bound};
use kayak_font::{CoordinateSystem, KayakFont};

use crate::core::{
    render_command::RenderCommand,
    styles::{Style, StyleProp},
    widget,
};

#[widget]
pub fn Text(size: f32, content: String, styles: Option<Style>, font: Option<String>) {
    let font_name = font;
    let font: Binding<Option<KayakFont>> =
        context.get_asset(font_name.clone().unwrap_or("Roboto".into()));

    context.bind(&font);

    // TODO: It might be worth caching the measurement here until content changes.
    let (layout_size, parent_size) =
        if let Some(parent_id) = context.widget_manager.get_valid_parent(parent_id.unwrap()) {
            if let Some(layout) = context.widget_manager.get_layout(&parent_id) {
                if let Some(font) = font.get() {
                    let measurement = font.measure(
                        CoordinateSystem::PositiveYDown,
                        &content,
                        size,
                        size * 1.2,
                        (layout.width, layout.height),
                    );

                    (measurement, (layout.width, layout.height))
                } else {
                    ((0.0, 0.0), (layout.width, layout.height))
                }
            } else {
                ((0.0, 0.0), (0.0, 0.0))
            }
        } else {
            ((0.0, 0.0), (0.0, 0.0))
        };

    let render_command = RenderCommand::Text {
        content: content.clone(),
        size,
        parent_size,
        font: font_name.clone().unwrap_or("Roboto".into()),
    };

    *styles = Some(Style {
        render_command: StyleProp::Value(render_command),
        width: StyleProp::Value(Units::Pixels(layout_size.0)),
        height: StyleProp::Value(Units::Pixels(layout_size.1)),
        ..styles.clone().unwrap_or_default()
    });
}
