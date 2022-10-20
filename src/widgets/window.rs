use bevy::prelude::{
    Bundle, Changed, Color, Commands, Component, Entity, In, Or, Query, Vec2, With,
};

use crate::{
    children::KChildren,
    context::{Mounted, WidgetName},
    event::{Event, EventType},
    event_dispatcher::EventDispatcherContext,
    on_event::OnEvent,
    prelude::WidgetContext,
    styles::{Corner, Edge, KStyle, PositionType, RenderCommand, StyleProp, Units},
    widget::Widget,
};

use super::{
    background::BackgroundBundle,
    clip::ClipBundle,
    text::{TextProps, TextWidgetBundle},
};

#[derive(Component, Debug, Default)]
pub struct KWindow {
    /// If true, allows the window to be draggable by its title bar
    pub draggable: bool,
    /// The position at which to display the window in pixels
    pub position: Vec2,
    /// The size of the window in pixels
    pub size: Vec2,
    /// The text to display in the window's title bar
    pub title: String,

    pub is_dragging: bool,
    pub offset: Vec2,
    pub title_bar_entity: Option<Entity>,
    // pub children: Vec<Entity>,
}

impl Widget for KWindow {}

#[derive(Bundle)]
pub struct WindowBundle {
    pub window: KWindow,
    pub styles: KStyle,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for WindowBundle {
    fn default() -> Self {
        Self {
            window: Default::default(),
            styles: Default::default(),
            children: Default::default(),
            widget_name: KWindow::default().get_name(),
        }
    }
}

pub fn window_update(
    In((widget_context, window_entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<
        (&mut KStyle, &KChildren, &mut KWindow),
        Or<(
            Changed<KWindow>,
            Changed<KStyle>,
            Changed<KChildren>,
            With<Mounted>,
        )>,
    >,
) -> bool {
    let mut has_changed = false;
    if let Ok((mut window_style, children, mut window)) = query.get_mut(window_entity) {
        *window_style = KStyle {
            background_color: StyleProp::Value(Color::rgba(0.125, 0.125, 0.125, 1.0)),
            border_color: StyleProp::Value(Color::rgba(0.0781, 0.0898, 0.101, 1.0)),
            border: StyleProp::Value(Edge::all(4.0)),
            border_radius: StyleProp::Value(Corner::all(5.0)),
            render_command: StyleProp::Value(RenderCommand::Quad),
            position_type: StyleProp::Value(PositionType::SelfDirected),
            left: StyleProp::Value(Units::Pixels(window.position.x)),
            top: StyleProp::Value(Units::Pixels(window.position.y)),
            width: StyleProp::Value(Units::Pixels(window.size.x)),
            height: StyleProp::Value(Units::Pixels(window.size.y)),
            min_width: StyleProp::Value(Units::Pixels(window.size.x)),
            min_height: StyleProp::Value(Units::Pixels(window.size.y)),
            ..window_style.clone()
        };

        if window.title_bar_entity.is_none() {
            let title = window.title.clone();

            let mut title_children = KChildren::new();
            // Spawn title children
            let title_entity = commands
                .spawn(TextWidgetBundle {
                    text: TextProps {
                        content: title.clone(),
                        size: 16.0,
                        line_height: Some(25.0),
                        ..Default::default()
                    },
                    styles: KStyle {
                        height: StyleProp::Value(Units::Pixels(25.0)),
                        ..KStyle::default()
                    },
                    ..Default::default()
                })
                .id();
            title_children.add(title_entity);

            let title_background_entity = commands
                .spawn(BackgroundBundle {
                    styles: KStyle {
                        render_command: StyleProp::Value(RenderCommand::Quad),
                        background_color: StyleProp::Value(Color::rgba(0.0781, 0.0898, 0.101, 1.0)),
                        border_radius: StyleProp::Value(Corner::all(5.0)),
                        height: StyleProp::Value(Units::Pixels(24.0)),
                        width: StyleProp::Value(Units::Stretch(1.0)),
                        left: StyleProp::Value(Units::Pixels(0.0)),
                        right: StyleProp::Value(Units::Pixels(0.0)),
                        top: StyleProp::Value(Units::Pixels(0.0)),
                        bottom: StyleProp::Value(Units::Pixels(0.0)),
                        padding_left: StyleProp::Value(Units::Pixels(5.0)),
                        ..KStyle::default()
                    },
                    children: title_children,
                    ..BackgroundBundle::default()
                })
                .id();
            window.title_bar_entity = Some(title_background_entity);

            if window.draggable {
                commands
                    .entity(window.title_bar_entity.unwrap())
                    .insert(OnEvent::new(
                        move |In((mut event_dispatcher_context, event, entity)): In<(
                            EventDispatcherContext,
                            Event,
                            Entity,
                        )>,
                              mut query: Query<&mut KWindow>| {
                            if let Ok(mut window) = query.get_mut(window_entity) {
                                match event.event_type {
                                    EventType::MouseDown(data) => {
                                        event_dispatcher_context.capture_cursor(entity);
                                        window.is_dragging = true;
                                        window.offset = Vec2::new(
                                            window.position.x - data.position.0,
                                            window.position.y - data.position.1,
                                        );
                                        window.title_bar_entity = None;
                                    }
                                    EventType::MouseUp(..) => {
                                        event_dispatcher_context.release_cursor(entity);
                                        window.is_dragging = false;
                                        window.title_bar_entity = None;
                                    }
                                    EventType::Hover(data) => {
                                        if window.is_dragging {
                                            window.position = Vec2::new(
                                                window.offset.x + data.position.0,
                                                window.offset.y + data.position.1,
                                            );
                                            window.title_bar_entity = None;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            (event_dispatcher_context, event)
                        },
                    ));
            }
            widget_context.add_widget(Some(window_entity), window.title_bar_entity.unwrap());

            let mut clip_bundle = ClipBundle {
                children: children.clone(),
                ..ClipBundle::default()
            };
            clip_bundle.styles.padding = StyleProp::Value(Edge::all(Units::Pixels(10.0)));

            let clip_entity = commands.spawn(clip_bundle).id();
            widget_context.add_widget(Some(window_entity), clip_entity);
            // let children = widget_context.get_children(window_entity);
            has_changed = true;
        }
    }

    has_changed
}
