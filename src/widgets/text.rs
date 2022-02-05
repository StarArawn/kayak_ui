use kayak_core::{styles::Units, Binding, Bound};
use kayak_font::{CoordinateSystem, KayakFont};

use crate::core::{
    render_command::RenderCommand,
    styles::{Style, StyleProp},
    widget, OnEvent, WidgetProps,
};

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct TextProps {
    pub content: String,
    pub font: Option<String>,
    pub line_height: Option<f32>,
    pub size: f32,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(OnEvent)]
    pub on_event: Option<OnEvent>,
    #[prop_field(Focusable)]
    pub focusable: Option<bool>,
}

#[widget]
pub fn Text(props: TextProps) {
    let TextProps {
        content,
        font,
        line_height,
        size,
        ..
    } = props.clone();
    let font_name = font;
    let font: Binding<Option<KayakFont>> =
        context.get_asset(font_name.clone().unwrap_or("Roboto".into()));

    context.bind(&font);

    // TODO: It might be worth caching the measurement here until content changes.
    let (layout_size, parent_size) =
        if let Some(parent_id) = context.get_valid_parent(parent_id.unwrap()) {
            if let Some(layout) = context.get_layout(&parent_id) {
                if let Some(font) = font.get() {
                    let measurement = font.measure(
                        CoordinateSystem::PositiveYDown,
                        &content,
                        size,
                        line_height.unwrap_or(size * 1.2),
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
        line_height: line_height.unwrap_or(size * 1.2),
        font: font_name.clone().unwrap_or("Roboto".into()),
    };

    props.styles = Some(Style {
        render_command: StyleProp::Value(render_command),
        width: StyleProp::Value(Units::Pixels(layout_size.0)),
        height: StyleProp::Value(Units::Pixels(layout_size.1)),
        ..props.styles.clone().unwrap_or_default()
    });
}
