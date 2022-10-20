use bevy::{
    prelude::{Bundle, Changed, Color, Commands, Component, Entity, In, Or, Query, With},
    window::CursorIcon,
};

use crate::{
    context::{Mounted, WidgetName},
    on_event::OnEvent,
    prelude::{KChildren, Units, WidgetContext},
    styles::{Corner, KCursorIcon, KStyle, RenderCommand, StyleProp},
    widget::Widget,
};

#[derive(Component, Default)]
pub struct KButton;

#[derive(Bundle)]
pub struct KButtonBundle {
    pub button: KButton,
    pub styles: KStyle,
    pub on_event: OnEvent,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for KButtonBundle {
    fn default() -> Self {
        Self {
            button: Default::default(),
            styles: Default::default(),
            on_event: Default::default(),
            children: KChildren::default(),
            widget_name: KButton::default().get_name(),
        }
    }
}

impl Widget for KButton {}

pub fn button_update(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    _: Commands,
    mut query: Query<(&mut KStyle, &KChildren), Or<(Changed<KButton>, With<Mounted>)>>,
) -> bool {
    if let Ok((mut style, children)) = query.get_mut(entity) {
        *style = KStyle::default()
            .with_style(KStyle {
                render_command: StyleProp::Value(RenderCommand::Quad),
                ..Default::default()
            })
            .with_style(style.clone())
            .with_style(KStyle {
                render_command: StyleProp::Value(RenderCommand::Quad),
                background_color: StyleProp::Value(Color::rgba(0.0781, 0.0898, 0.101, 1.0)),
                border_radius: StyleProp::Value(Corner::all(5.0)),
                height: StyleProp::Value(Units::Pixels(45.0)),
                padding_left: StyleProp::Value(Units::Stretch(1.0)),
                padding_right: StyleProp::Value(Units::Stretch(1.0)),
                padding_bottom: StyleProp::Value(Units::Stretch(1.0)),
                padding_top: StyleProp::Value(Units::Stretch(1.0)),
                cursor: StyleProp::Value(KCursorIcon(CursorIcon::Hand)),
                ..Default::default()
            });

        children.process(&widget_context, Some(entity));

        return true;
    }

    false
}
