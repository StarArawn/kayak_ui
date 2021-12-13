use bevy::{
    math::Vec2,
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::{WindowDescriptor, Windows},
    PipelinedDefaultPlugins,
};
use bevy_kayak_ui::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_core::{
    styles::{Style, StyleProp, Units},
    Bound, EventType, Index, MutableBound, OnEvent,
};
use kayak_ui::components::App;
use kayak_ui::core::{rsx, widget};
use kayak_widgets::{Button, Text, Window};

#[widget]
fn Counter(context: &mut KayakContext) {
    let text_styles = Style {
        bottom: StyleProp::Value(Units::Stretch(1.0)),
        left: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        top: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Pixels(26.0)),
        ..Default::default()
    };

    let count = context.create_state(0i32).unwrap();
    let cloned_count = count.clone();
    let on_event = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click => {
            cloned_count.set(cloned_count.get() + 1);
        }
        _ => {}
    });

    let count_value = count.get();
    rsx! {
        <>
            <Window position={(50.0, 50.0)} size={(300.0, 300.0)} title={"Counter Example".to_string()}>
                <Text size={32.0} content={format!("Current Count: {}", count_value).to_string()}>{}</Text>
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
