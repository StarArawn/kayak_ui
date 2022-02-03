use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::Index;
use kayak_ui::core::WidgetProps;
use kayak_ui::core::{render, rsx, widget};
use kayak_ui::widgets::{App, Inspector, Window};

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct Props {}

#[widget]
fn CustomWidget(props: Props) {
    rsx! {
        <>
            <Window draggable={true} position={(50.0, 50.0)} size={(300.0, 300.0)} title={"Window 1".to_string()}>
                {}
            </Window>
            <Window draggable={true} position={(550.0, 50.0)} size={(200.0, 200.0)} title={"Window 2".to_string()}>
                {}
            </Window>
        </>
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add("Roboto", asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <CustomWidget />
                <Inspector />
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
