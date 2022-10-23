use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

use crate::TodoList;

#[derive(Component, Default, Clone, PartialEq)]
pub struct TodoInputProps {
    has_focus: bool,
}

impl Widget for TodoInputProps {}
impl WidgetProps for TodoInputProps {}

#[derive(Bundle)]
pub struct TodoInputBundle {
    pub widget: TodoInputProps,
    pub focusable: Focusable,
    pub styles: KStyle,
    pub widget_name: WidgetName,
}

impl Default for TodoInputBundle {
    fn default() -> Self {
        Self {
            widget: TodoInputProps::default(),
            focusable: Default::default(),
            styles: KStyle {
                render_command: StyleProp::Value(RenderCommand::Layout),
                // height: StyleProp::Value(Units::Stretch(1.0)),
                height: StyleProp::Value(Units::Auto),
                width: StyleProp::Value(Units::Stretch(1.0)),
                bottom: StyleProp::Value(Units::Pixels(20.0)),
                ..KStyle::default()
            },
            widget_name: TodoInputProps::default().get_name(),
        }
    }
}

pub fn render_todo_input(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    mut todo_list: ResMut<TodoList>,
    keyboard_input: Res<Input<KeyCode>>,
    change_query: Query<Entity, (With<TodoInputProps>, With<Mounted>)>,
    prop_query: Query<&TodoInputProps>,
) -> bool {
    if todo_list.is_changed() || !change_query.is_empty() {
        if let Ok(props) = prop_query.get(entity) {
            let on_change = OnChange::new(
                move |In((_widget_context, _, value)): In<(WidgetContext, Entity, String)>,
                      mut todo_list: ResMut<TodoList>| {
                    todo_list.new_item = value;
                },
            );

            if keyboard_input.just_pressed(KeyCode::Return) {
                if props.has_focus {
                    let value = todo_list.new_item.clone();
                    todo_list.items.push(value);
                    todo_list.new_item.clear();
                }
            }

            let handle_click = OnEvent::new(
                move |In((event_dispatcher_context, _, event, _)): In<(
                    EventDispatcherContext,
                    WidgetState,
                    Event,
                    Entity,
                )>,
                      mut todo_list: ResMut<TodoList>| {
                    match event.event_type {
                        EventType::Click(..) => {
                            let value = todo_list.new_item.clone();
                            todo_list.items.push(value);
                            todo_list.new_item.clear();
                        }
                        _ => {}
                    }
                    (event_dispatcher_context, event)
                },
            );

            let handle_focus = OnEvent::new(
                move |In((event_dispatcher_context, _, event, _)): In<(
                    EventDispatcherContext,
                    WidgetState,
                    Event,
                    Entity,
                )>,
                      mut props_query: Query<&mut TodoInputProps>| {
                    if let Ok(mut props) = props_query.get_mut(entity) {
                        match event.event_type {
                            EventType::Focus => {
                                props.has_focus = true;
                            }
                            EventType::Blur => {
                                props.has_focus = false;
                            }
                            _ => {}
                        }
                    }
                    (event_dispatcher_context, event)
                },
            );

            let parent_id = Some(entity);
            rsx! {
                <ElementBundle
                    id={"element_bundle"}
                    styles={KStyle {
                        layout_type: StyleProp::Value(LayoutType::Row),
                        height: StyleProp::Value(Units::Pixels(32.0)),
                        cursor: StyleProp::Value(KCursorIcon(CursorIcon::Text)),
                        ..Default::default()
                    }}
                    on_event={handle_focus}
                >
                    {
                        // You can spawn whatever you want on UI widgets this way! :)
                        commands.entity(element_bundle).insert(Focusable);
                    }
                    <TextBoxBundle
                        styles={KStyle {
                            bottom: StyleProp::Value(Units::Stretch(1.0)),
                            top: StyleProp::Value(Units::Stretch(1.0)),
                            ..Default::default()
                        }}
                        text_box={TextBoxProps {
                            value: todo_list.new_item.clone(),
                            placeholder: Some("Add item here..".into()),
                            ..Default::default()
                        }}
                        on_change={on_change}
                    />
                    <KButtonBundle
                        styles={KStyle {
                            width: StyleProp::Value(Units::Pixels(32.0)),
                            height: StyleProp::Value(Units::Pixels(32.0)),
                            left: StyleProp::Value(Units::Pixels(5.0)),
                            ..Default::default()
                        }}
                        on_event={handle_click}
                    >
                        <TextWidgetBundle
                            text={TextProps {
                                content: "+".into(),
                                ..Default::default()
                            }}
                        />
                    </KButtonBundle>
                </ElementBundle>
            }
            return true;
        }
    }
    false
}
