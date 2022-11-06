use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

use crate::TodoList;

#[derive(Component, Default, Clone, PartialEq)]
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
                height: StyleProp::Value(Units::Auto),
                width: StyleProp::Value(Units::Stretch(1.0)),
                ..KStyle::default()
            },
            widget_name: TodoItemsProps::default().get_name(),
        }
    }
}

pub fn render_todo_items(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    todo_list: Res<TodoList>,
) -> bool {
    let parent_id = Some(entity);
    rsx! {
        <ElementBundle
            styles={KStyle {
                height: Units::Auto.into(),
                ..Default::default()
            }}
        >
            {todo_list.items.iter().enumerate().for_each(|(index, content)| {
                let handle_click = OnEvent::new(
                    move |In((event_dispatcher_context, _, event, _)): In<(
                        EventDispatcherContext,
                        WidgetState,
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
                            background_color: Color::rgba(0.160, 0.172, 0.235, 1.0).into(),
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
                                user_styles: KStyle {
                                    right: StyleProp::Value(Units::Stretch(1.0)),
                                    top: StyleProp::Value(Units::Stretch(1.0)),
                                    bottom: StyleProp::Value(Units::Stretch(1.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }}
                        />
                        <KButtonBundle
                            button={KButton {
                                text: "X".into(),
                                user_styles: KStyle {
                                    width: StyleProp::Value(Units::Pixels(32.0)),
                                    height: StyleProp::Value(Units::Pixels(32.0)),
                                    left: StyleProp::Value(Units::Pixels(15.0)),
                                    ..Default::default()
                                }
                            }}
                            on_event={handle_click}
                        />
                    </ElementBundle>
                }
            })}
        </ElementBundle>
    }
    true
}
