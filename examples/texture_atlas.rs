use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, KStyle, *};

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    let camera_entity = commands
        .spawn((Camera2dBundle::default(), CameraUIKayak))
        .id();

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let image_handle = asset_server.load("texture_atlas.png");

    //texture_atlas.png uses 16 pixel sprites and is 272x128 pixels
    let tile_size = 16;
    let columns = 272 / tile_size;
    let rows = 128 / tile_size;
    let atlas = bevy::sprite::TextureAtlas::from_grid(
        image_handle.clone(),
        bevy::prelude::Vec2::splat(tile_size as f32),
        columns,
        rows,
        None,
        None,
    );

    //The sign in the top right of the image would be index 16
    let sign_index = 16;
    //The flower is in the 6(-1) row and 15 collumn
    let flower_index = columns * 5 + 15;

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;

    let atlas_styles = KStyle {
        position_type: StyleProp::Value(KPositionType::ParentDirected),
        width: StyleProp::Value(Units::Pixels(200.0)),
        height: StyleProp::Value(Units::Pixels(200.0)),
        ..KStyle::default()
    };

    let rect = atlas.textures[sign_index];
    let sign_position = rect.min;
    let sign_size = rect.max - rect.min;

    let rect = atlas.textures[flower_index];
    let flower_position = rect.min;
    let flower_size = rect.max - rect.min;

    rsx! {
        <KayakAppBundle>
            <TextureAtlasBundle
                atlas={TextureAtlasProps {
                    handle: image_handle.clone(),
                    position: sign_position,
                    tile_size: sign_size,
                }}
                styles={atlas_styles.clone()}
            />
            <TextureAtlasBundle
                atlas={TextureAtlasProps {
                    handle: image_handle,
                    position: flower_position,
                    tile_size: flower_size,
                }}
                styles={atlas_styles}
            />
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((KayakContextPlugin, KayakWidgets))
        .add_systems(Startup, startup)
        .run()
}
