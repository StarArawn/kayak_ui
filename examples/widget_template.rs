/// This is a simple widget template.
/// It'll auto update if the props change.
/// Don't forget to register the update system with kayak!
use bevy::prelude::*;
use kayak_ui::prelude::*;

#[derive(Component, Default)]
pub struct WidgetProps;

impl Widget for WidgetProps {}

#[derive(Bundle)]
pub struct WidgetBundle {
    pub widget: WidgetProps,
    pub styles: KStyle,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for WidgetBundle {
    fn default() -> Self {
        Self {
            widget: WidgetProps::default(),
            styles: KStyle {
                render_command: StyleProp::Value(RenderCommand::Clip),
                height: StyleProp::Value(Units::Stretch(1.0)),
                width: StyleProp::Value(Units::Stretch(1.0)),
                ..KStyle::default()
            },
            children: KChildren::default(),
            widget_name: WidgetProps::default().get_name(),
        }
    }
}

pub fn update_widget(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    _: Commands,
    mut query: Query<
        (&Style, &KChildren),
        Or<(Changed<Style>, Changed<WidgetProps>, With<Mounted>)>,
    >,
) -> bool {
    if let Ok((_, children)) = query.get_mut(entity) {
        children.process(&widget_context, Some(entity));
        return true;
    }
    false
}

fn main() {}
