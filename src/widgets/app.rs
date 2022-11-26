use bevy::prelude::*;
use kayak_ui_macros::rsx;

use crate::{
    children::KChildren,
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle, RenderCommand, StyleProp, Units},
    widget::{EmptyState, Widget, WidgetParam},
    CameraUIKayak,
};

use super::ClipBundle;

#[derive(Component, Default, Clone, PartialEq, Eq)]
pub struct KayakApp;

impl Widget for KayakApp {}

/// Kayak's default root widget
/// This widget provides a width/height that matches the screen size in bevy
/// It will auto update if bevy's window changes as well.
#[derive(Bundle)]
pub struct KayakAppBundle {
    pub app: KayakApp,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for KayakAppBundle {
    fn default() -> Self {
        Self {
            app: Default::default(),
            styles: Default::default(),
            computed_styles: ComputedStyles::default(),
            children: Default::default(),
            widget_name: KayakApp::default().get_name(),
        }
    }
}

pub fn app_update(
    In((widget_context, entity, previous_props_entity)): In<(KayakWidgetContext, Entity, Entity)>,
    widget_param: WidgetParam<KayakApp, EmptyState>,
    camera: Query<&Camera, With<CameraUIKayak>>,
    windows: Res<Windows>,
) -> bool {
    let mut window_change = false;

    if let Ok(app_style) = widget_param.computed_style_query.get(entity) {
        if let Some(camera_entity) = widget_context.camera_entity {
            if let Ok(camera) = camera.get(camera_entity) {
                if let Some(size) = camera.logical_viewport_size() {
                    if app_style.0.width != StyleProp::Value(Units::Pixels(size.x)) {
                        window_change = true;
                    }
                    if app_style.0.height != StyleProp::Value(Units::Pixels(size.y)) {
                        window_change = true;
                    }
                } else {
                    let primary_window = windows.get_primary().unwrap();
                    if app_style.0.width != StyleProp::Value(Units::Pixels(primary_window.width()))
                    {
                        window_change = true;
                    }
                    if app_style.0.height
                        != StyleProp::Value(Units::Pixels(primary_window.height()))
                    {
                        window_change = true;
                    }
                }
            }
        }
    }

    widget_param.has_changed(&widget_context, entity, previous_props_entity) || window_change
}

/// TODO: USE CAMERA INSTEAD OF WINDOW!!
pub fn app_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<(&KStyle, &mut ComputedStyles, &KChildren)>,
    camera: Query<&Camera, With<CameraUIKayak>>,
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
) -> bool {
    let (mut width, mut height) = (0.0, 0.0);

    if let Some(camera_entity) = widget_context.camera_entity {
        if let Ok(camera) = camera.get(camera_entity) {
            if let Some(size) = camera.logical_viewport_size() {
                width = size.x;
                height = size.y;
            } else if let Some(viewport) = camera
                .target
                .get_render_target_info(&windows, &images)
                .as_ref()
                .map(|target_info| {
                    let scale = target_info.scale_factor;
                    (target_info.physical_size.as_dvec2() / scale).as_vec2()
                })
            {
                width = viewport.x;
                height = viewport.y;
            }
        }
    }

    if let Ok((app_style, mut computed_styles, children)) = query.get_mut(entity) {
        *computed_styles = KStyle::default()
            .with_style(KStyle {
                render_command: RenderCommand::Layout.into(),
                width: Units::Pixels(width).into(),
                height: Units::Pixels(height).into(),
                ..Default::default()
            })
            .with_style(app_style)
            .into();

        let parent_id = Some(entity);
        rsx! {
            <ClipBundle
                children={children.clone()}
            />
        }
    }

    true
}
