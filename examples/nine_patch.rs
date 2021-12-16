use bevy::{
    math::Vec2,
    prelude::{App as BevyApp, AssetServer, Commands, Handle, Res, ResMut},
    window::{WindowDescriptor, Windows},
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, ImageManager, UICameraBundle};
use kayak_ui::core::{
    layout_cache::Space,
    render,
    styles::{Style, StyleProp, Units},
    Index,
};
use kayak_widgets::{App, NinePatch};

fn startup(
    mut commands: Commands,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    mut image_manager: ResMut<ImageManager>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    let window_size = if let Some(window) = windows.get_primary() {
        Vec2::new(window.width(), window.height())
    } else {
        panic!("Couldn't find primary window!");
    };

    let handle: Handle<bevy::render::texture::Image> = asset_server.load("panel.png");
    let ui_image_handle = image_manager.get(&handle);

    let context = BevyContext::new(window_size.x, window_size.y, |styles, context| {
        // The border prop splits up the image into 9 quadrants like so:
        // 1----2----3
        // |         |
        // 4    9    5
        // |         |
        // 6----7----8
        // The sizes of sprites for a 15 pixel border are as follows:
        // TopLeft = (15, 15)
        // TopRight = (15, 15)
        // LeftCenter = (15, image_height)
        // RightCenter = (15, image_height)
        // TopCenter = (image_width, 15)
        // BottomCenter = (image_width, 15)
        // BottomRight = (15, 15)
        // BottomLeft = (15, 15)
        // Middle = (
        // 30 being left border + right border
        //   image_width - 30
        // 30 being top border + bottom border
        //   image_height - 30
        // )
        //

        let nine_patch_styles = Style {
            width: StyleProp::Value(Units::Pixels(512.0)),
            height: StyleProp::Value(Units::Pixels(512.0)),
            ..Style::default()
        };

        render! {
            <App styles={Some(styles.clone())}>
                <NinePatch
                    styles={Some(nine_patch_styles)}
                    border={Space {
                        left: 15.0,
                        right: 15.0,
                        top: 15.0,
                        bottom: 15.0,
                    }}
                    handle={ui_image_handle}
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
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
}
