use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    commands.spawn(UICameraBundle::new());

    let mut widget_context = KayakRootContext::new();
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <TextWidgetBundle
                text={TextProps {
                    content: "Hello World".into(),
                    ..Default::default()
                }}
            />
        </KayakAppBundle>
    }
    commands.insert_resource(widget_context);
}

fn main() {
    App::new()
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
