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

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
<<<<<<< HEAD
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <TodoApp />
            </App>
        }
    });

    commands.insert_resource(context);
}

fn main() {
    BevyApp::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("UI Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
=======
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    commands.spawn(UICameraBundle::new());

    let mut widget_context = Context::new();
    widget_context.add_widget_system(TodoItemsProps::default().get_name(), update_todo_items);
    widget_context.add_widget_system(TodoInputProps::default().get_name(), update_todo_input);
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
>>>>>>> exp/main
}
