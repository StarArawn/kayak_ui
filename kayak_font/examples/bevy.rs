use bevy::{
    math::Vec2,
    prelude::{App as BevyApp, AssetServer, Commands, Handle, Res},
    render2::{camera::OrthographicCameraBundle, color::Color},
    window::WindowDescriptor,
    PipelinedDefaultPlugins,
};
use kayak_font::{KayakFont, KayakFontPlugin};

mod renderer;
use renderer::FontRenderPlugin;
use renderer::Text;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let font_handle: Handle<KayakFont> = asset_server.load("roboto.kayak_font");

    commands
        .spawn()
        .insert(Text {
            color: Color::WHITE,
            content: "Hello World!".into(),
            font_size: 32.0,
            position: Vec2::new(5.0, 5.0),
            size: Vec2::new(100.0, 100.0),
        })
        .insert(font_handle);
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
        .add_plugin(FontRenderPlugin)
        .add_startup_system(startup)
        .run();
}
