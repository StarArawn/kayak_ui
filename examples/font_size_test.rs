use bevy::prelude::*;
use kayak_ui::{
    prelude::{widgets::*, *},
    CameraUIKayak,
};

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    let camera_entity = commands
        .spawn(Camera2dBundle::default())
        .insert(CameraUIKayak)
        .id();

    font_mapping.set_default(asset_server.load("fonts/roboto.kttf"));
    font_mapping.force_subpixel(&asset_server.load("fonts/roboto.kttf"));

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <TransitionBundle
                transition={TransitionProps {
                    easing: TransitionEasing::SineInOut,
                    timeout: 5000.0,
                    looping: true,
                    style_a: KStyle {
                        font_size: 10.0.into(),
                        ..Default::default()
                    },
                    style_b: KStyle {
                        font_size: 1000.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }}
            >
                <TextWidgetBundle
                    text={TextProps {
                        content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Cras sed tellus neque. Proin tempus ligula a mi molestie aliquam.".into(),
                        ..Default::default()
                    }}
                />
            </TransitionBundle>
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((KayakContextPlugin, KayakWidgets))
        .add_systems(Startup, startup)
        .run()
}
