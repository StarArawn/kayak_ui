//! This example demonstrates how to use a [on_layout](kayak_core::WidgetProps::get_on_layout)
//! event in widgets.
//!
//! The problem here is strictly contrived for example purposes.
//! We use grow/shrink buttons to set the value of a `width` bound to an [crate::Background] element's width
//! On change of layout we print current width of that element and update the text of Width label.
use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_core::{
    styles::{Edge, LayoutType, Style, StyleProp, Units},
    OnLayout,
};
use kayak_core::{Color, EventType, OnEvent};
use kayak_render_macros::use_state;
use kayak_ui::widgets::{App, Element, Text, Window};
use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    widgets::Button,
};
use kayak_ui::{
    core::{render, rsx, widget},
    widgets::Background,
};

/// This widget provides a theme to its children
#[widget]
fn GrowShrink() {
    // This is width of background element we update via buttons
    let (background_width, set_width, _) = use_state!(150.0);

    let panel_style = Style {
        layout_type: StyleProp::Value(LayoutType::Row),
        width: StyleProp::Value(Units::Auto),
        height: StyleProp::Value(Units::Pixels(50.0)),
        offset: StyleProp::Value(Edge::all(Units::Pixels(10.0))),
        ..Default::default()
    };

    // Grow/Shrink button styles
    let button_styles = Style {
        width: StyleProp::Value(Units::Pixels(100.0)),
        height: StyleProp::Value(Units::Pixels(30.0)),
        background_color: StyleProp::Value(Color::new(0.33, 0.33, 0.33, 1.0)),
        offset: StyleProp::Value(Edge::all(Units::Pixels(10.0))),
        ..Default::default()
    };

    // The background style of element growing/shrink
    let fill = Style {
        width: StyleProp::Value(Units::Pixels(background_width)),
        height: StyleProp::Value(Units::Pixels(28.0)),
        layout_type: StyleProp::Value(LayoutType::Column),
        background_color: StyleProp::Value(Color::new(1.0, 0.0, 0.0, 1.0)),
        ..Default::default()
    };

    // Cloned function for use in closures
    let grow_fn = set_width.clone();
    let shrink_fn = set_width.clone();

    let grow = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click(..) => grow_fn(background_width + rand::random::<f32>() * 10.0),
        _ => {}
    });

    let shrink = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click(..) => shrink_fn(background_width - rand::random::<f32>() * 10.0),
        _ => {}
    });

    // layout width will be used by width label which we update `on_layout`
    let (layout_width, set_layout_width, _) = use_state!(0.0);

    let update_text = OnLayout::new(move |_, layout_event| {
        println!("Layout changed! New width = {}", layout_event.layout.width);
        set_layout_width(layout_event.layout.width);
    });

    rsx! {
        <>
            <Window position={(100.0, 100.0)} size={(400.0, 400.0)} title={"Grow/Shrink Example".to_string()}>
                <Text size={25.0} content={format!("Width: {:?}", layout_width).to_string()} />
                <Element styles={Some(panel_style)}>
                    <Button styles={Some(button_styles)} on_event={Some(grow)}>
                        <Text size={20.0} content={"Grow".to_string()}/>
                    </Button>
                    <Button styles={Some(button_styles)} on_event={Some(shrink)}>
                        <Text size={20.0} content={"Shrink".to_string()}/>
                    </Button>
                </Element>
                <Background styles={Some(fill)} on_layout={Some(update_text)} />
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
