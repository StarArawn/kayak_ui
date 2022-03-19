use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_core::{styles::{LayoutType, Style, StyleProp, Units}, OnLayout};
use kayak_core::{Color, EventType, OnEvent};
use kayak_render_macros::use_state;
use kayak_ui::core::{render, rsx, widget, Index};
use kayak_ui::widgets::{App, Text, Window};
use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    widgets::Button,
};


#[widget]
fn GrowShrink() {
    let (width, set_width, _) = use_state!(150.0);

    let button_styles = Style {
        width: StyleProp::Value(Units::Pixels(100.0)),
        height: StyleProp::Value(Units::Pixels(24.0)),
        layout_type: StyleProp::Value(LayoutType::Row),
        background_color: StyleProp::Value(Color::new(0.33, 0.33, 0.33, 1.0)),
        ..Default::default()
    };

    let fill = Style {
        width: StyleProp::Value(Units::Pixels(width)),
        height: StyleProp::Value(Units::Pixels(28.0)),
        layout_type: StyleProp::Value(LayoutType::Column),
        background_color: StyleProp::Value(Color::new(1.0, 0.0, 0.0, 1.0)),
        ..Default::default()
    };

    let grow_fn = set_width.clone();
    let shrink_fn = set_width.clone();

    let grow = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click(..) =>  grow_fn(width + 10.0),
        _ => {},
    });

    let shrink = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click(..) => shrink_fn(width - 10.0),
        _ => {},
    });

    let (layout_width, set_layout_width, _) = use_state!(0.0);

    let update_text = OnLayout::new(move |_, layout_event| {        
        layout_event.layout.width *= 2.0;
        println!("Width = {}", layout_event.layout.width);
        set_layout_width(layout_event.layout.width);
    });

    rsx! {
        <>
            <Window position={(100.0, 100.0)} size={(400.0, 400.0)} title={"Grow Example".to_string()}>
                <Text size={25.0} content={format!("Width: {:?}", layout_width).to_string()} />
                <Button styles={Some(button_styles)} on_event={Some(grow)}>
                    <Text size={20.0} content={"Grow".to_string()}/>
                </Button>
                <Button styles={Some(button_styles)} on_event={Some(shrink)}>
                    <Text size={20.0} content={"Shrink".to_string()}/>
                </Button>
                <Button styles={Some(fill)} on_layout={Some(update_text)}>
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
                <GrowShrink />
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
