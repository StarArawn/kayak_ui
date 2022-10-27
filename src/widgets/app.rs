use bevy::prelude::*;
use morphorm::Units;

use crate::{
    children::KChildren,
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{KStyle, RenderCommand, StyleProp},
    widget::{EmptyState, Widget, WidgetParam},
};

#[derive(Component, Default, Clone, PartialEq)]
pub struct KayakApp;

impl Widget for KayakApp {}

/// Kayak's default root widget
/// This widget provides a width/height that matches the screen size in bevy
/// It will auto update if bevy's window changes as well.
#[derive(Bundle)]
pub struct KayakAppBundle {
    pub app: KayakApp,
    pub styles: KStyle,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for KayakAppBundle {
    fn default() -> Self {
        Self {
            app: Default::default(),
            styles: Default::default(),
            children: Default::default(),
            widget_name: KayakApp::default().get_name(),
        }
    }
}

pub fn app_update(
    In((widget_context, entity, previous_props_entity)): In<(KayakWidgetContext, Entity, Entity)>,
    windows: Res<Windows>,
    widget_param: WidgetParam<KayakApp, EmptyState>,
) -> bool {
    let primary_window = windows.get_primary().unwrap();

    let mut window_change = false;
    if let Ok(app_style) = widget_param.style_query.get(entity) {
        if app_style.width != StyleProp::Value(Units::Pixels(primary_window.width())) {
            window_change = true;
        }
        if app_style.height != StyleProp::Value(Units::Pixels(primary_window.height())) {
            window_change = true;
        }
    }

    widget_param.has_changed(&widget_context, entity, previous_props_entity) || window_change
}

/// TODO: USE CAMERA INSTEAD OF WINDOW!!
pub fn app_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    _: Commands,
    windows: Res<Windows>,
    mut query: Query<(&mut KStyle, &KChildren)>,
) -> bool {
    let primary_window = windows.get_primary().unwrap();
    if let Ok((mut app_style, children)) = query.get_mut(entity) {
        if app_style.width != StyleProp::Value(Units::Pixels(primary_window.width())) {
            app_style.width = StyleProp::Value(Units::Pixels(primary_window.width()));
        }
        if app_style.height != StyleProp::Value(Units::Pixels(primary_window.height())) {
            app_style.height = StyleProp::Value(Units::Pixels(primary_window.height()));
        }
        app_style.render_command = StyleProp::Value(RenderCommand::Layout);
        children.process(&widget_context, Some(entity));
    }

    true
}
