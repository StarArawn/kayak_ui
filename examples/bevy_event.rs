use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, EventReader, EventWriter, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{
    render, rsx,
    styles::{Style, StyleProp, Units},
    widget, EventType, OnEvent,
};
use kayak_ui::widgets::{App, Button, Text, Window};

pub struct MyEvent;

#[widget]
fn EventWindow() {
    let button_text_styles = Style {
        left: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };

    let on_event = OnEvent::new(move |ctx, event| match event.event_type {
        EventType::Click(..) => {
            ctx.query_world::<EventWriter<MyEvent>, _, ()>(|mut writer| writer.send(MyEvent));
        }
        _ => {}
    });

    rsx! {
        <>
            <Window draggable={true} position={(50.0, 50.0)} size={(300.0, 300.0)} title={"Bevy Event Example".to_string()}>
                <Button on_event={Some(on_event)}>
                    <Text styles={Some(button_text_styles)} line_height={Some(40.0)} size={24.0} content={"Send bevy event".to_string()}>{}</Text>
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

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <EventWindow />
            </App>
        }
    });

    commands.insert_resource(context);
}

fn on_my_event(mut reader: EventReader<MyEvent>) {
    for _ in reader.iter() {
        println!("MyEvent detected");
    }
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
        .add_event::<MyEvent>()
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .add_system(on_my_event)
        .run();
}
