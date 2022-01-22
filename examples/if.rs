use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{
    render, rsx,
    styles::{Style, StyleProp, Units},
    widget, Bound, EventType, Index, MutableBound, OnEvent,
};
use kayak_ui::widgets::{App, Button, If, Text, Window};

#[widget]
fn Removal(context: &mut KayakContext) {
    let text_styles = Style {
        bottom: StyleProp::Value(Units::Stretch(1.0)),
        left: StyleProp::Value(Units::Stretch(0.1)),
        right: StyleProp::Value(Units::Stretch(0.1)),
        top: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };

    let is_visible = context.create_state(true).unwrap();
    let cloned_is_visible = is_visible.clone();
    let on_event = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click(..) => {
            cloned_is_visible.set(!cloned_is_visible.get());
        }
        _ => {}
    });

    let is_visible = is_visible.get();
    rsx! {
        <>
            <Window position={(50.0, 50.0)} size={(300.0, 300.0)} title={"If Example".to_string()}>
                <If condition={is_visible}>
                    <Text styles={Some(text_styles)} size={32.0} content={"Hello!".to_string()} />
                </If>
                <Button on_event={Some(on_event)}>
                    <Text line_height={Some(40.0)} size={24.0} content={"Swap!".to_string()} />
                </Button>
            </Window>
        </>
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add("Roboto", asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <Removal />
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
