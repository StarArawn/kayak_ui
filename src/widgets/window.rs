use bevy::{
    prelude::{Bundle, Color, Commands, Component, Entity, In, Query, Vec2, Res, ResMut},
    window::CursorIcon,
};
use kayak_ui_macros::rsx;

use crate::{
    children::KChildren,
    context::WidgetName,
    event::{EventType, KEvent},
    event_dispatcher::EventDispatcherContext,
    on_event::OnEvent,
    prelude::KayakWidgetContext,
    styles::{
        ComputedStyles, Corner, Edge, KCursorIcon, KPositionType, KStyle, RenderCommand, StyleProp,
        Units,
    },
    widget::Widget,
    Focusable,
};

use super::{
    background::BackgroundBundle,
    clip::ClipBundle,
    text::{TextProps, TextWidgetBundle},
    window_context_provider::WindowContext,
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
    /// Styles for the main window quad.
    pub window_styles: KStyle,
    /// A set of styles to apply to the children element wrapper.
    pub children_styles: KStyle,
}

#[derive(Component, PartialEq, Clone, Debug, Default)]
pub struct KWindowState {
    pub is_dragging: bool,
    pub offset: Vec2,
    pub position: Vec2,
    pub focused: bool,
}

impl Widget for KWindow {}

/// Default window widget
/// A simple widget that renders a window.
/// Does not support much customization.
#[derive(Bundle)]
pub struct WindowBundle {
    pub window: KWindow,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for WindowBundle {
    fn default() -> Self {
        Self {
            window: Default::default(),
            styles: Default::default(),
            computed_styles: ComputedStyles::default(),
            children: Default::default(),
            widget_name: KWindow::default().get_name(),
        }
    }
}

pub fn window_render(
    In(window_entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    mut query: Query<(&KStyle, &mut ComputedStyles, &KChildren, &KWindow)>,
    state_query: Query<&KWindowState>,
    mut context_query: Query<&mut WindowContext>,
) -> bool {
    if let Ok((window_style, mut computed_styles, window_children, window)) =
        query.get_mut(window_entity)
    {
        let possible_context_entity =
            widget_context.get_context_entity::<WindowContext>(window_entity);
        let z_index = if let Some(window_context_entity) = possible_context_entity {
            if let Ok(mut window_context) = context_query.get_mut(window_context_entity) {
                Some(window_context.get_or_add(window_entity))
            } else {
                None
            }
        } else {
            None
        };

        *computed_styles = KStyle::default()
            .with_style(KStyle {
                position_type: StyleProp::Value(KPositionType::SelfDirected),
                z_index: if let Some(z_index) = z_index {
                    StyleProp::Value(z_index as i32 * 1000)
                } else {
                    StyleProp::Default
                },
                ..Default::default()
            })
            .with_style(window_style)
            .into();

        let title = window.title.clone();

        let state_entity = widget_context.use_state(
            &mut commands,
            window_entity,
            KWindowState {
                position: window.initial_position,
                offset: Vec2::ZERO,
                is_dragging: false,
                focused: false,
            },
        );

        if let Ok(state) = state_query.get(state_entity) {
            let parent_id = Some(window_entity);

            let focus_event = OnEvent::new(
                    move |In(_entity): In<Entity>,
                    mut event: ResMut<KEvent>,
                      mut query: Query<&mut KWindowState>,
                      mut context_query: Query<&mut WindowContext>| {
                    if let Ok(mut window) = query.get_mut(state_entity) {
                        event.stop_propagation();
                        event.prevent_default();
                        match event.event_type {
                            EventType::Focus => {
                                window.focused = true;
                                if let Some(window_context_entity) = possible_context_entity {
                                    if let Ok(mut context) =
                                        context_query.get_mut(window_context_entity)
                                    {
                                        context.shift_to_top(window_entity);
                                    }
                                }
                            }
                            EventType::Blur => {
                                window.focused = false;
                                window.is_dragging = false;
                            }
                            _ => {}
                        }
                    }
                },
            );

            rsx! {
                <ElementBundle
                    id={"window_entity"}
                    styles={window.window_styles.clone().with_style(KStyle {
                        background_color: StyleProp::Value(Color::rgba(0.188, 0.203, 0.274, 1.0)),
                        border_color: StyleProp::Value(if state.focused { Color::rgba(0.933, 0.745, 0.745, 1.0) } else { Color::rgba(0.239, 0.258, 0.337, 1.0) }),
                        border: StyleProp::Value(Edge::all(2.0)),
                        border_radius: StyleProp::Value(Corner::all(10.0)),
                        render_command: StyleProp::Value(RenderCommand::Quad),
                        position_type: StyleProp::Value(KPositionType::SelfDirected),
                        left: StyleProp::Value(Units::Pixels(state.position.x)),
                        top: StyleProp::Value(Units::Pixels(state.position.y)),
                        width: StyleProp::Value(Units::Pixels(window.size.x)),
                        height: StyleProp::Value(Units::Pixels(window.size.y)),
                        min_width: StyleProp::Value(Units::Pixels(window.size.x)),
                        min_height: StyleProp::Value(Units::Pixels(window.size.y)),
                        ..Default::default()
                    })}
                    on_event={focus_event}
                >
                    {commands.entity(window_entity).insert(Focusable)}
                    <BackgroundBundle
                        id={"title_bar_entity"}
                        styles={KStyle {
                            cursor: StyleProp::Value(KCursorIcon(CursorIcon::Hand)),
                            render_command: StyleProp::Value(RenderCommand::Quad),
                            background_color: StyleProp::Value(Color::rgba(0.188, 0.203, 0.274, 1.0)),
                            border_radius: Corner::all(10.0).into(),
                            height: StyleProp::Value(Units::Pixels(24.0)),
                            width: StyleProp::Value(Units::Stretch(1.0)),
                            left: StyleProp::Value(Units::Pixels(0.0)),
                            right: StyleProp::Value(Units::Pixels(0.0)),
                            top: StyleProp::Value(Units::Pixels(0.0)),
                            bottom: StyleProp::Value(Units::Pixels(0.0)),
                            padding_left: StyleProp::Value(Units::Pixels(5.0)),
                            padding_top: Units::Stretch(1.0).into(),
                            padding_bottom: Units::Stretch(1.0).into(),
                            ..KStyle::default()
                        }}
                    >
                        <TextWidgetBundle
                            styles={KStyle {
                                top: Units::Stretch(1.0).into(),
                                bottom: Units::Stretch(1.0).into(),
                                ..Default::default()
                            }}
                            text={TextProps {
                                content: title,
                                size: 14.0,
                                ..Default::default()
                            }}
                        />
                    </BackgroundBundle>
                    {
                        // This code needs to go after the closing tag for the background bundle as that is when the
                        // widget is "spawned". Adding this code after just the starting tag will cause this OnEvent
                        // to be wiped out with a default version.
                        if window.draggable {
                            commands
                                .entity(title_bar_entity)
                                .insert(OnEvent::new(
                                    move |In(entity): In<Entity>,
                                    mut event_dispatcher_context: ResMut<EventDispatcherContext>,
                                    mut event: ResMut<KEvent>,
                                        mut query: Query<&mut KWindowState>| {
                                        if let Ok(mut window) = query.get_mut(state_entity) {
                                            event.prevent_default();
                                            event.stop_propagation();
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
                                    },
                                ));
                        }
                    }
                    <BackgroundBundle
                        styles={KStyle {
                            background_color: StyleProp::Value(Color::rgba(0.239, 0.258, 0.337, 1.0)),
                            width: Units::Stretch(1.0).into(),
                            height: Units::Pixels(2.0).into(),
                            ..Default::default()
                        }}
                    />
                    <ClipBundle
                        styles={window.children_styles.clone().with_style(KStyle {
                            top: Units::Pixels(10.0).into(),
                            left: Units::Pixels(10.0).into(),
                            right: Units::Pixels(10.0).into(),
                            bottom: Units::Pixels(10.0).into(),
                            ..Default::default()
                        })}
                        children={window_children.clone()}
                    />
                </ElementBundle>
            };
        }
    }

    true
}
