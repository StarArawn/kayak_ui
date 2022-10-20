use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

use crate::TodoList;

#[derive(Component, Default)]
pub struct TodoItemsProps;

impl Widget for TodoItemsProps {}

#[derive(Bundle)]
pub struct TodoItemsBundle {
    pub widget: TodoItemsProps,
    pub styles: KStyle,
    pub widget_name: WidgetName,
}

impl Default for TodoItemsBundle {
    fn default() -> Self {
        Self {
            widget: TodoItemsProps::default(),
            styles: KStyle {
                render_command: StyleProp::Value(RenderCommand::Layout),
                // height: StyleProp::Value(Units::Stretch(1.0)),
                width: StyleProp::Value(Units::Stretch(1.0)),
                ..KStyle::default()
            },
            widget_name: TodoItemsProps::default().get_name(),
        }
    }
}

pub fn update_todo_items(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    todo_list: Res<TodoList>,
    query: Query<&TodoItemsProps, Or<(Changed<Style>, Changed<TodoItemsProps>, With<Mounted>)>>,
) -> bool {
    if query.is_empty() || todo_list.is_changed() {
        let parent_id = Some(entity);
        rsx! {
            <ElementBundle>
                {todo_list.items.iter().enumerate().for_each(|(index, content)| {
                    let handle_click = OnEvent::new(
                        move |In((event_dispatcher_context, event, _)): In<(
                            EventDispatcherContext,
                            Event,
                            Entity,
                        )>,
                            mut todo_list: ResMut<TodoList>,| {
                            match event.event_type {
                                EventType::Click(..) => {
                                    todo_list.items.remove(index);
                                },
                                _ => {}
                            }
                            (event_dispatcher_context, event)
                        },
                    );
                    constructor! {
                        <ElementBundle
                            styles={KStyle {
                                render_command: StyleProp::Value(RenderCommand::Quad),
                                background_color: StyleProp::Value(Color::rgba(0.0781, 0.0898, 0.101, 1.0)),
                                border_radius: StyleProp::Value(Corner::all(3.0)),
                                bottom: StyleProp::Value(Units::Pixels(5.0)),
                                height: StyleProp::Value(Units::Auto),
                                padding: StyleProp::Value(Edge::all(Units::Pixels(10.0))),
                                layout_type: StyleProp::Value(LayoutType::Row),
                                ..Default::default()
                            }}
                        >
                            <TextWidgetBundle
                                text={TextProps {
                                    content: content.clone(),
                                    ..Default::default()
                                }}
                                styles={KStyle {
                                    right: StyleProp::Value(Units::Stretch(1.0)),
                                    top: StyleProp::Value(Units::Stretch(1.0)),
                                    bottom: StyleProp::Value(Units::Stretch(1.0)),
                                    ..Default::default()
                                }}
                            />
                            <KButtonBundle
                                styles={KStyle {
                                    width: StyleProp::Value(Units::Pixels(32.0)),
                                    height: StyleProp::Value(Units::Pixels(32.0)),
                                    left: StyleProp::Value(Units::Pixels(15.0)),
                                    ..Default::default()
                                }}
                                on_event={handle_click}
                            >
                                <TextWidgetBundle text={TextProps {
                                    content: "X".into(),
                                    ..Default::default()
                                }} />
                            </KButtonBundle>
                        </ElementBundle>
                    }
                })}
            </ElementBundle>
        }
        return true;
    }
    false
}
