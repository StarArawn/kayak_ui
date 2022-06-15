use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Handle, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_core::styles::PositionType;
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, ImageManager, UICameraBundle};
use kayak_ui::core::{
    render,
    styles::{Corner, Style, StyleProp, Units},
};
use kayak_ui::widgets::{App, Image};

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut image_manager: ResMut<ImageManager>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    let handle: Handle<bevy::render::texture::Image> = asset_server.load("generic-rpg-vendor.png");
    let ui_image_handle = image_manager.get(&handle);

    let context = BevyContext::new(|context| {
        let image_styles = Style {
            position_type: StyleProp::Value(PositionType::SelfDirected),
            left: StyleProp::Value(Units::Pixels(10.0)),
            top: StyleProp::Value(Units::Pixels(10.0)),
            border_radius: StyleProp::Value(Corner::all(500.0)),
            width: StyleProp::Value(Units::Pixels(200.0)),
            height: StyleProp::Value(Units::Pixels(182.0)),
            ..Style::default()
        };

        render! {
            <App>
                <Image styles={Some(image_styles)} handle={ui_image_handle} />
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
