use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut, State, SystemSet},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{
    render, rsx, widget, Event, EventType, Index, KayakContextRef, KeyCode, OnEvent,
};
use kayak_ui::widgets::{App, Text};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    MainMenu,
    Options,
    Play,
}

fn swap(mut state: ResMut<State<GameState>>) {
    if *state.current() == GameState::MainMenu {
        let _ = state.set(GameState::Options);
    } else if *state.current() == GameState::Options {
        let _ = state.set(GameState::Play);
    } else {
        let _ = state.set(GameState::MainMenu);
    }
}

fn handle_input(context: &mut KayakContextRef, event: &mut Event) {
    match event.event_type {
        EventType::KeyDown(event) => {
            if event.key() == KeyCode::Space {
                context.query_world::<ResMut<State<GameState>>, _, _>(swap);
            }
        }
        _ => {}
    };
}

#[widget]
fn StateSwitcher() {
    rsx! {
        <Text content={"Press space to switch states!".to_string()} size={32.0} />
    }
}

fn create_main_menu(mut commands: Commands) {
    let context = BevyContext::new(|context| {
        render! {
            <App on_event={Some(OnEvent::new(handle_input))}>
                <Text content={"Main Menu".to_string()} size={32.0} />
                <StateSwitcher />
            </App>
        }
    });

    commands.insert_resource(context);
}

fn create_options_menu(mut commands: Commands) {
    let context = BevyContext::new(|context| {
        render! {
            <App on_event={Some(OnEvent::new(handle_input))}>
                <Text content={"Options".to_string()} size={32.0} />
                <StateSwitcher />
            </App>
        }
    });

    commands.insert_resource(context);
}

fn create_play_menu(mut commands: Commands) {
    let context = BevyContext::new(|context| {
        render! {
            <App on_event={Some(OnEvent::new(handle_input))}>
                <Text content={"Play".to_string()} size={32.0} />
                <StateSwitcher />
            </App>
        }
    });

    commands.insert_resource(context);
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));
}

fn destroy(mut commands: Commands) {
    commands.remove_resource::<BevyContext>();
}
fn main() {
    BevyApp::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("UI Example"),
            ..Default::default()
        })
        .add_state(GameState::MainMenu)
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .add_system_set(SystemSet::on_enter(GameState::MainMenu).with_system(create_main_menu))
        .add_system_set(SystemSet::on_exit(GameState::MainMenu).with_system(destroy))
        .add_system_set(SystemSet::on_enter(GameState::Options).with_system(create_options_menu))
        .add_system_set(SystemSet::on_exit(GameState::Options).with_system(destroy))
        .add_system_set(SystemSet::on_enter(GameState::Play).with_system(create_play_menu))
        .add_system_set(SystemSet::on_exit(GameState::Play).with_system(destroy))
        .run();
}
