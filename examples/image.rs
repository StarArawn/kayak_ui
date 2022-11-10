use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, KStyle, *};

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    commands.spawn(UICameraBundle::new());

    let image = asset_server.load("generic-rpg-vendor.png");

    let mut widget_context = KayakRootContext::new();
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <KImageBundle
                image={KImage(image)}
                styles={KStyle {
                    position_type: StyleProp::Value(KPositionType::SelfDirected),
                    left: StyleProp::Value(Units::Pixels(10.0)),
                    top: StyleProp::Value(Units::Pixels(10.0)),
                    border_radius: StyleProp::Value(Corner::all(500.0)),
                    width: StyleProp::Value(Units::Pixels(200.0)),
                    height: StyleProp::Value(Units::Pixels(182.0)),
                    ..Default::default()
                }}
            />
        </KayakAppBundle>
    }
    commands.insert_resource(widget_context);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
