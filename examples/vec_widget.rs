use bevy::{
    math::Vec2,
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::{WindowDescriptor, Windows},
    DefaultPlugins,
};
use kayak_core::constructor;
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{render, widget, Index, VecTracker};
use kayak_widgets::{App, Text, Window};

fn startup(
    mut commands: Commands,
    windows: Res<Windows>,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add(asset_server.load("roboto.kayak_font"));

    let window_size = if let Some(window) = windows.get_primary() {
        Vec2::new(window.width(), window.height())
    } else {
        panic!("Couldn't find primary window!");
    };

    let context = BevyContext::new(window_size.x, window_size.y, |styles, context| {
        let data = vec!["Text1", "Text2", "Text3", "Text4"];

        render! {
            <App styles={Some(styles.clone())}>
                {VecTracker::new(
                    data.iter().map(|data| constructor! {
                        <Text content={data.clone().to_string()} size={16.0} />
                    }).collect::<Vec<_>>()
                )}
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
