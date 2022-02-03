use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{
    render, rsx, WidgetProps,
    styles::{Style, StyleProp, Units},
    use_state, widget, EventType, Index, OnEvent,
};
use kayak_ui::widgets::{App, Button, Text, Window};

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct CounterProps {}

#[widget]
fn Counter(props: CounterProps) {
    let text_styles = Style {
        bottom: StyleProp::Value(Units::Stretch(1.0)),
        left: StyleProp::Value(Units::Stretch(0.1)),
        right: StyleProp::Value(Units::Stretch(0.1)),
        top: StyleProp::Value(Units::Stretch(1.0)),
        width: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Pixels(28.0)),
        ..Default::default()
    };

    let button_text_styles = Style {
        left: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };

    let (count, set_count, ..) = use_state!(0i32);
    let on_event = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click(..) => set_count(count + 1),
        _ => {}
    });

    rsx! {
        <>
            <Window draggable={true} position={(50.0, 50.0)} size={(300.0, 300.0)} title={"Counter Example".to_string()}>
                <Text styles={Some(text_styles)} size={32.0} content={format!("Current Count: {}", count).to_string()}>{}</Text>
                <Button on_event={Some(on_event)}>
                    <Text styles={Some(button_text_styles)} line_height={Some(40.0)} size={24.0} content={"Count!".to_string()}>{}</Text>
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
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
}
