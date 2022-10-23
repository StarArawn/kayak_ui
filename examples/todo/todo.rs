use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

mod input;
mod items;

use crate::input::*;
use items::*;

// A bit of state management.
// Consider this like "global" state.
#[derive(Resource)]
pub struct TodoList {
    pub new_item: String,
    pub items: Vec<String>,
}

impl TodoList {
    pub fn new() -> Self {
        Self {
            new_item: "".into(),
            items: vec![
                "Buy milk".into(),
                "Paint Shed".into(),
                "Eat Dinner".into(),
                "Write new Bevy UI library".into(),
            ],
        }
    }
}

// Our own version of widget_update that handles resource change events.
pub fn widget_update_with_resource<
    Props: WidgetProps + PartialEq + Component + Clone,
    State: PartialEq + Component + Clone,
>(
    In((widget_context, entity, previous_entity)): In<(WidgetContext, Entity, Entity)>,
    todo_list: Res<TodoList>,
    widget_param: WidgetParam<Props, State>,
) -> bool {
    widget_param.has_changed(&widget_context, entity, previous_entity) || todo_list.is_changed()
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    commands.spawn(UICameraBundle::new());

    let mut widget_context = Context::new();
    widget_context.add_widget_data::<TodoItemsProps, EmptyState>();
    widget_context.add_widget_data::<TodoInputProps, EmptyState>();

    widget_context.add_widget_system(
        TodoItemsProps::default().get_name(),
        widget_update_with_resource::<TodoItemsProps, EmptyState>,
        render_todo_items,
    );
    widget_context.add_widget_system(
        TodoInputProps::default().get_name(),
        widget_update_with_resource::<TodoInputProps, EmptyState>,
        render_todo_input,
    );
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <WindowBundle
                window={KWindow {
                    title: "Todo App".into(),
                    draggable: true,
                    position: Vec2::new((1280.0 / 2.0) - (350.0 / 2.0), (720.0 / 2.0) - (600.0 / 2.0)),
                    size: Vec2::new(400.0, 600.0),
                    ..Default::default()
                }}
            >
                <TodoInputBundle />
                <ScrollContextProviderBundle>
                    <ScrollBoxBundle>
                        <TodoItemsBundle />
                    </ScrollBoxBundle>
                </ScrollContextProviderBundle>
            </WindowBundle>
        </KayakAppBundle>
    }
    commands.insert_resource(widget_context);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ContextPlugin)
        .add_plugin(KayakWidgets)
        .insert_non_send_resource(TodoList::new())
        .add_startup_system(startup)
        .run()
}
