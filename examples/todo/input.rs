use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

use crate::TodoList;

#[derive(Component, Default, Clone, PartialEq, Eq)]
pub struct TodoInputProps;

impl Widget for TodoInputProps {}

#[derive(Bundle)]
pub struct TodoInputBundle {
    pub widget: TodoInputProps,
    pub focusable: Focusable,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub widget_name: WidgetName,
}

impl Default for TodoInputBundle {
    fn default() -> Self {
        Self {
            widget: TodoInputProps::default(),
            focusable: Default::default(),
            styles: KStyle::default(),
            computed_styles: ComputedStyles(KStyle {
                render_command: StyleProp::Value(RenderCommand::Layout),
                // height: StyleProp::Value(Units::Stretch(1.0)),
                height: StyleProp::Value(Units::Auto),
                width: StyleProp::Value(Units::Stretch(1.0)),
                bottom: StyleProp::Value(Units::Pixels(20.0)),
                ..KStyle::default()
            }),
            widget_name: TodoInputProps::default().get_name(),
        }
    }
}

pub fn render_todo_input(
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    todo_list: Res<TodoList>,
) -> bool {
    let on_change = OnChange::new(
        move |In((_widget_context, _, value)): In<(KayakWidgetContext, Entity, String)>,
              mut todo_list: ResMut<TodoList>| {
            todo_list.new_item = value;
        },
    );

    let handle_click = OnEvent::new(
        move |In((event_dispatcher_context, _, event, _)): In<(
            EventDispatcherContext,
            WidgetState,
            KEvent,
            Entity,
        )>,
              mut todo_list: ResMut<TodoList>| {
            match event.event_type {
                EventType::Click(..) => {
                    if !todo_list.new_item.is_empty() {
                        let value = todo_list.new_item.clone();
                        todo_list.items.push(value);
                        todo_list.new_item.clear();
                    }
                }
                _ => {}
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
        >
            {
                // You can spawn whatever you want on UI widgets this way! :)
                commands.entity(element_bundle).insert(Focusable);
            }
            <TextBoxBundle
                styles={KStyle {
                    // bottom: StyleProp::Value(Units::Stretch(1.0)),
                    // top: StyleProp::Value(Units::Stretch(1.0)),
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
                button={KButton {
                    text: "+".into(),
                }}
                on_event={handle_click}
            />
        </ElementBundle>
    };
    true
}
