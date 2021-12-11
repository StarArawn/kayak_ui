use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Handle, Res},
    window::WindowDescriptor,
    PipelinedDefaultPlugins,
};
use kayak_font::{KayakFont, KayakFontPlugin};

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle: Handle<KayakFont> = asset_server.load("roboto.kayak_font");
    dbg!(font_handle);
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
        .add_plugin(KayakFontPlugin)
        .add_startup_system(startup)
        .run();
}
