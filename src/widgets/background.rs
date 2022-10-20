use bevy::prelude::{Bundle, Changed, Commands, Component, Entity, In, Or, Query, With};

use crate::{
    children::KChildren,
    context::{Mounted, WidgetName},
    on_event::OnEvent,
    prelude::WidgetContext,
    styles::{KStyle, RenderCommand, StyleProp},
    widget::Widget,
};

#[derive(Component, Default)]
pub struct Background;

impl Widget for Background {}

#[derive(Bundle)]
pub struct BackgroundBundle {
    pub background: Background,
    pub styles: KStyle,
    pub children: KChildren,
    pub on_event: OnEvent,
    pub widget_name: WidgetName,
}

impl Default for BackgroundBundle {
    fn default() -> Self {
        Self {
            background: Default::default(),
            styles: Default::default(),
            children: Default::default(),
            on_event: Default::default(),
            widget_name: Background::default().get_name(),
        }
    }
}

pub fn update_background(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    _: Commands,
    mut query: Query<
        (&mut KStyle, &KChildren),
        Or<(
            (Changed<KStyle>, Changed<KChildren>, With<Background>),
            With<Mounted>,
        )>,
    >,
) -> bool {
    if let Ok((mut style, children)) = query.get_mut(entity) {
        style.render_command = StyleProp::Value(RenderCommand::Quad);
        children.process(&widget_context, Some(entity));
        return true;
    }
    false
}
