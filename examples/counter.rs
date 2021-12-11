use bevy::{
    math::Vec2,
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::{WindowDescriptor, Windows},
    PipelinedDefaultPlugins,
};
use bevy_kayak_ui::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_components::{Button, Text, Window};
use kayak_core::{
    context::KayakContext,
    styles::{Style, StyleProp, Units},
    EventType, Index, OnEvent,
};
use kayak_ui::components::App;
use kayak_ui::core::{rsx, widget};

#[widget]
fn Counter(context: &mut KayakContext) {
    let count = {
        let x = context.create_state(0i32).unwrap();
        *x
    };
    let text_styles = Style {
        bottom: StyleProp::Value(Units::Stretch(1.0)),
        left: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        top: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Pixels(26.0)),
        ..Default::default()
    };

    let id = self.get_id();
    let on_event = OnEvent::new(move |context, event| match event.event_type {
        EventType::Click => {
            context.set_current_id(id);
            context.set_state(count + 1);
        }
        _ => {}
    });

    rsx! {
        <>
            <Window position={(50.0, 50.0)} size={(300.0, 300.0)} title={"Counter Example".to_string()}>
                <Text size={32.0} content={format!("Current Count: {}", count).to_string()}>{}</Text>
                <Button on_event={Some(on_event)}>
                    <Text styles={Some(text_styles)} size={24.0} content={"Count!".to_string()}>{}</Text>
                </Button>
            </Window>
        </>
    }
}

fn startup(
    mut commands: Commands,
    windows: Res<Windows>,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add(asset_server.load("roboto.kayak_font"));

    let window_size = if let Some(window) = windows.get_primary() {
        Vec2::new(window.width(), window.height())
    } else {
        panic!("Couldn't find primary window!");
    };

    let context = BevyContext::new(window_size.x, window_size.y, |styles, context| {
        // Hack to trick the proc macro for right now..
        let parent_id: Option<Index> = None;
        rsx! {
            <App styles={Some(styles.clone())}>
                <Counter />
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
        .add_plugins(PipelinedDefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
}
