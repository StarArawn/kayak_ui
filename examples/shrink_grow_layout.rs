use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_core::{styles::{LayoutType, Style, StyleProp, Units}, OnLayout};
use kayak_core::{Color, EventType, OnEvent};
use kayak_ui::core::{bind, render, rsx, widget, Binding, MutableBound, Bound, Index};
use kayak_ui::widgets::{App, Text, Window};
use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    widgets::Button,
};
#[derive(Clone, PartialEq)]
struct WidthCounter(pub f32);

#[widget]
fn GrowShrink() {
    let bar_width =
        context.query_world::<Res<Binding<WidthCounter>>, _, _>(move |width| width.clone());

    context.bind(&bar_width);

    let width = bar_width.get().0;

    let button_styles = Style {
        width: StyleProp::Value(Units::Pixels(100.0)),
        height: StyleProp::Value(Units::Pixels(24.0)),
        layout_type: StyleProp::Value(LayoutType::Row),
        background_color: StyleProp::Value(Color::new(0.33, 0.33, 0.33, 1.0)),
        ..Default::default()
    };

    let fill = Style {
        width: StyleProp::Value(Units::Pixels(150.0)),
        height: StyleProp::Value(Units::Pixels(24.0)),
        layout_type: StyleProp::Value(LayoutType::Column),
        background_color: StyleProp::Value(Color::new(1.0, 0.0, 0.0, 1.0)),
        ..Default::default()
    };

    let grow = OnEvent::new(move |context, event| match event.event_type {
        EventType::Click(..) => 
            context.query_world::<Res<Binding<WidthCounter>>, _, _>(move |width| width.set(WidthCounter(width.get().0 + 10.0))),
        _ => {},
    });

    let shrink = OnEvent::new(move |context, event| match event.event_type {
        EventType::Click(..) => 
            context.query_world::<Res<Binding<WidthCounter>>, _, _>(move |width| width.set(WidthCounter(width.get().0 - 10.0))),
        _ => {},
    });

    let resize = OnLayout::new(move |context, layout_event| {
        context.query_world::<Res<Binding<WidthCounter>>, _, _>(move |width| width.clone());
        layout_event.layout.width = width;
    });

    rsx! {
        <>
            <Window position={(100.0, 100.0)} size={(400.0, 400.0)} title={"Grow Example".to_string()}>
                <Text size={32.0} content={format!("Width: {}", width).to_string()}>{}</Text>
                <Button styles={Some(button_styles)} on_event={Some(grow)}>
                    <Text size={20.0} content={"Grow".to_string()}/>
                </Button>
                <Button styles={Some(button_styles)} on_event={Some(shrink)}>
                    <Text size={20.0} content={"Shrink".to_string()}/>
                </Button>
                <Button styles={Some(fill)} on_layout={Some(resize)}/>
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

    commands.insert_resource(bind(WidthCounter(0f32)));

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
