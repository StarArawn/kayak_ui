use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{constructor, render, Index, VecTracker};
use kayak_ui::widgets::{App, Text};

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add("Roboto", asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        let data = vec![
            "Text 1", "Text 2", "Text 3", "Text 4", "Text 5", "Text 6", "Text 7", "Text 8",
            "Text 9", "Text 10",
        ];
        render! {
            <App>
                {VecTracker::from(data.iter().map(|data| {
                    constructor! {
                        <Text content={data.clone().to_string()} size={16.0} />
                    }
                }))}
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
