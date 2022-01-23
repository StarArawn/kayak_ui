use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};

use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    core::{render, Index},
    widgets::{App, Text},
};

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // Font mapping maps bevy kayak fonts to kayak.
    // We need this because we use bevy asset system to load things in!
    mut font_mapping: ResMut<FontMapping>,
) {
    // You can load in any kayak fonts and they are key'd by a String.
    font_mapping.add("Roboto", asset_server.load("roboto.kayak_font"));

    // Add the bevy kayak ui camera:
    commands.spawn_bundle(UICameraBundle::new());

    // Now we create the kayak context which is wrapped in a BevyContext resource.
    let context = BevyContext::new(|context| {
        // Using render we can create our widgets. Here we have an `App` which is recommended as the
        // base widget that should be used. With the bevy feature it automatically sizes everything to
        // scale with the bevy window.
        render! {
            <App>
                <Text size={100.0} content={"Hello World".to_string()} />
            </App>
        }
    });

    // Finally insert the context into bevy as a resource.
    commands.insert_resource(context);
}
fn main() {
    BevyApp::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Hello World!"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
}
