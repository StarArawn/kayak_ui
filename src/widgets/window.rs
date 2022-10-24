use bevy::{
    prelude::{Bundle, Color, Commands, Component, Entity, In, Query, Vec2},
    window::CursorIcon,
};
use kayak_ui_macros::rsx;

use crate::{
    children::KChildren,
    context::WidgetName,
    event::{Event, EventType},
    event_dispatcher::EventDispatcherContext,
    on_event::OnEvent,
    prelude::WidgetContext,
    styles::{Corner, Edge, KCursorIcon, KStyle, PositionType, RenderCommand, StyleProp, Units},
    widget::Widget,
    widget_state::WidgetState,
};

use super::{
    background::BackgroundBundle,
    clip::ClipBundle,
    text::{TextProps, TextWidgetBundle},
    ElementBundle,
};

#[derive(Component, PartialEq, Clone, Debug, Default)]
pub struct KWindow {
    /// If true, allows the window to be draggable by its title bar
    pub draggable: bool,
    /// The initial position at which to display the window in pixels
    pub initial_position: Vec2,
    /// The size of the window in pixels
    pub size: Vec2,
    /// The text to display in the window's title bar
    pub title: String,
}

#[derive(Component, PartialEq, Clone, Debug, Default)]
pub struct KWindowState {
    pub is_dragging: bool,
    pub offset: Vec2,
    pub position: Vec2,
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

pub fn window_render(
    In((widget_context, window_entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<(&KStyle, &KChildren, &KWindow)>,
    state_query: Query<&KWindowState>,
) -> bool {
    if let Ok((window_style, window_children, window)) = query.get_mut(window_entity) {
        let title = window.title.clone();

        let state_entity = widget_context.use_state(
            &mut commands,
            window_entity,
            KWindowState {
                position: window.initial_position,
                offset: Vec2::ZERO,
                is_dragging: false,
            },
        );

        if let Ok(state) = state_query.get(state_entity) {
            let parent_id = Some(window_entity);
            rsx! {
                <ElementBundle
                    styles={KStyle {
                        background_color: StyleProp::Value(Color::rgba(0.125, 0.125, 0.125, 1.0)),
                        border_color: StyleProp::Value(Color::rgba(0.0781, 0.0898, 0.101, 1.0)),
                        border: StyleProp::Value(Edge::all(4.0)),
                        border_radius: StyleProp::Value(Corner::all(5.0)),
                        render_command: StyleProp::Value(RenderCommand::Quad),
                        position_type: StyleProp::Value(PositionType::SelfDirected),
                        left: StyleProp::Value(Units::Pixels(state.position.x)),
                        top: StyleProp::Value(Units::Pixels(state.position.y)),
                        width: StyleProp::Value(Units::Pixels(window.size.x)),
                        height: StyleProp::Value(Units::Pixels(window.size.y)),
                        min_width: StyleProp::Value(Units::Pixels(window.size.x)),
                        min_height: StyleProp::Value(Units::Pixels(window.size.y)),
                        ..window_style.clone()
                    }}
                >
                    <BackgroundBundle
                        id={"title_bar_entity"}
                        styles={KStyle {
                            cursor: StyleProp::Value(KCursorIcon(CursorIcon::Hand)),
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
                        }}
                    >
                        <TextWidgetBundle
                            text={TextProps {
                                content: title.clone(),
                                size: 14.0,
                                line_height: Some(25.0),
                                ..Default::default()
                            }}
                            styles={KStyle {
                                height: StyleProp::Value(Units::Pixels(25.0)),
                                ..KStyle::default()
                            }}
                        />
                    </BackgroundBundle>
                    {
                        if window.draggable {
                            commands
                                .entity(title_bar_entity)
                                .insert(OnEvent::new(
                                    move |In((mut event_dispatcher_context, _, event, entity)): In<(
                                        EventDispatcherContext,
                                        WidgetState,
                                        Event,
                                        Entity,
                                    )>,
                                        mut query: Query<&mut KWindowState>| {
                                        if let Ok(mut window) = query.get_mut(state_entity) {
                                            match event.event_type {
                                                EventType::MouseDown(data) => {
                                                    event_dispatcher_context.capture_cursor(entity);
                                                    window.is_dragging = true;
                                                    window.offset = Vec2::new(
                                                        window.position.x - data.position.0,
                                                        window.position.y - data.position.1,
                                                    );
                                                }
                                                EventType::MouseUp(..) => {
                                                    event_dispatcher_context.release_cursor(entity);
                                                    window.is_dragging = false;
                                                }
                                                EventType::Hover(data) => {
                                                    if window.is_dragging {
                                                        window.position = Vec2::new(
                                                            window.offset.x + data.position.0,
                                                            window.offset.y + data.position.1,
                                                        );
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        (event_dispatcher_context, event)
                                    },
                                ));
                        }
                    }
                    <ClipBundle
                        styles={KStyle {
                            padding: StyleProp::Value(Edge::all(Units::Pixels(10.0))),
                            ..Default::default()
                        }}
                        children={window_children.clone()}
                    />
                </ElementBundle>
            }
        }
    }

    true
}
