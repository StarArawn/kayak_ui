use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Handle, Res, ResMut},
    render::texture::ImageSettings,
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_core::styles::PositionType;
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, ImageManager, UICameraBundle};
use kayak_ui::core::{
    render,
    styles::{Style, StyleProp, Units},
};
use kayak_ui::widgets::{App, TextureAtlas};

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut image_manager: ResMut<ImageManager>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    let image_handle: Handle<bevy::render::texture::Image> = asset_server.load("texture_atlas.png");
    let ui_image_handle = image_manager.get(&image_handle);

    //texture_atlas.png uses 16 pixel sprites and is 272x128 pixels
    let tile_size = 16;
    let columns = 272 / tile_size;
    let rows = 128 / tile_size;
    let atlas = bevy::sprite::TextureAtlas::from_grid(
        image_handle,
        bevy::prelude::Vec2::splat(tile_size as f32),
        columns,
        rows,
    );

    //The sign in the top right of the image would be index 16
    let sign_index = 16;
    //The flower is in the 6(-1) row and 15 collumn
    let flower_index = columns * 5 + 15;

    let context = BevyContext::new(|context| {
        let atlas_styles = Style {
            position_type: StyleProp::Value(PositionType::ParentDirected),
            width: StyleProp::Value(Units::Pixels(200.0)),
            height: StyleProp::Value(Units::Pixels(200.0)),
            ..Style::default()
        };

        let rect = atlas.textures[sign_index];
        let sign_position = rect.min;
        let sign_size = rect.max - rect.min;

        let rect = atlas.textures[flower_index];
        let flower_position = rect.min;
        let flower_size = rect.max - rect.min;

        render! {
            <App>
                <TextureAtlas styles={Some(atlas_styles)}
                handle={ui_image_handle}
                position={(sign_position.x, sign_position.y)}
                tile_size={(sign_size.x, sign_size.y)}
                 />
                <TextureAtlas styles={Some(atlas_styles)}
                handle={ui_image_handle}
                position={(flower_position.x, flower_position.y)}
                tile_size={(flower_size.x, flower_size.y)}
                 />
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
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
}
