use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{
    render, rsx,
    styles::{LayoutType, Style, StyleProp, Units},
    use_state, widget, Bound, EventType, Handler, Index, MutableBound, OnEvent,
};
use kayak_ui::widgets::{App, Element, OnChange, TextBox, Window};

mod add_button;
mod card;
mod cards;
mod delete_button;
use add_button::AddButton;
use cards::Cards;

#[derive(Debug, Clone, PartialEq)]
pub struct Todo {
    name: String,
}

#[widget]
fn TodoApp() {
    let (todos, set_todos) = use_state!(vec![
        Todo {
            name: "Use bevy to make a game!".to_string(),
        },
        Todo {
            name: "Help contribute to bevy!".to_string(),
        },
        Todo {
            name: "Join the bevy discord!".to_string(),
        },
    ]);

    let (new_todo_value, set_new_todo_value) = use_state!("".to_string());

    let text_box_styles = Style {
        right: StyleProp::Value(Units::Pixels(10.0)),
        ..Style::default()
    };

    let top_area_styles = Style {
        layout_type: StyleProp::Value(LayoutType::Row),
        bottom: StyleProp::Value(Units::Pixels(10.0)),
        height: StyleProp::Value(Units::Pixels(30.0)),
        padding_top: StyleProp::Value(Units::Stretch(1.0)),
        padding_bottom: StyleProp::Value(Units::Stretch(1.0)),
        ..Style::default()
    };

    let on_change = OnChange::new(move |event| {
        set_new_todo_value(event.value);
    });

    let new_todo_value_cloned = new_todo_value.clone();
    let mut todos_cloned = todos.clone();
    let cloned_set_todos = set_todos.clone();
    let add_events = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click => {
            if !new_todo_value_cloned.is_empty() {
                todos_cloned.push(Todo {
                    name: new_todo_value_cloned.clone(),
                });
                cloned_set_todos(todos_cloned.clone());
            }
        }
        _ => {}
    });

    let mut todos_cloned = todos.clone();
    let cloned_set_todos = set_todos.clone();
    let handle_delete = Handler::new(move |card_id: usize| {
        todos_cloned.remove(card_id);
        cloned_set_todos(todos_cloned.clone());
    });

    rsx! {
        <Window position={(415.0, 50.0)} size={(450.0, 600.0)} title={"Todo!".to_string()}>
            <Element styles={Some(top_area_styles)}>
                <TextBox
                    styles={Some(text_box_styles)}
                    value={new_todo_value}
                    placeholder={Some("Type here to add a new todo!".to_string())}
                    on_change={Some(on_change)}
                />
                <AddButton on_event={Some(add_events)} />
            </Element>
            <Cards cards={todos} on_delete={handle_delete} />
        </Window>
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add(asset_server.load("roboto.kayak_font"));

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
}
