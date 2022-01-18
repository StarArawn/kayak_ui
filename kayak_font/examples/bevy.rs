use bevy::{
    math::Vec2,
    prelude::{App as BevyApp, AssetServer, Commands, Handle, Res},
    render::{camera::OrthographicCameraBundle, color::Color},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_font::{bevy::KayakFontPlugin, Alignment, KayakFont};

mod renderer;
use renderer::FontRenderPlugin;
use renderer::Text;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let font_handle: Handle<KayakFont> = asset_server.load("roboto.kayak_font");

    commands
        .spawn()
        .insert(Text {
            horz_alignment: Alignment::Start,
            color: Color::WHITE,
            content: "Hello World! This text should wrap because its super long!".into(),
            font_size: 32.0,
            line_height: 32.0 * 1.2, // Firefox method of calculating default line heights see: https://developer.mozilla.org/en-US/docs/Web/CSS/line-height
            position: Vec2::new(5.0, 5.0),
            size: Vec2::new(250.0, 100.0),
        })
        .insert(font_handle.clone());

    commands
        .spawn()
        .insert(Text {
            horz_alignment: Alignment::End,
            color: Color::WHITE,
            content: "This is some text that will wrap and also be aligned to the right.".into(),
            font_size: 32.0,
            line_height: 32.0 * 1.2, // Firefox method of calculating default line heights see: https://developer.mozilla.org/en-US/docs/Web/CSS/line-height
            position: Vec2::new(-255.0, 5.0),
            size: Vec2::new(250.0, 100.0),
        })
        .insert(font_handle.clone());

    commands
        .spawn()
        .insert(Text {
            horz_alignment: Alignment::Middle,
            color: Color::WHITE,
            content: "This is some text that will wrap and also be aligned in the middle.".into(),
            font_size: 32.0,
            line_height: 32.0 * 1.2, // Firefox method of calculating default line heights see: https://developer.mozilla.org/en-US/docs/Web/CSS/line-height
            position: Vec2::new(-125.0, -155.0),
            size: Vec2::new(250.0, 100.0),
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
        .add_plugins(DefaultPlugins)
        .add_plugin(KayakFontPlugin)
        .add_plugin(FontRenderPlugin)
        .add_startup_system(startup)
        .run();
}
