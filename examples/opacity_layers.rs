use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    let camera_entity = commands
        .spawn((Camera2dBundle::default(), CameraUIKayak))
        .id();

    font_mapping.set_default(asset_server.load("lato-light.kttf"));

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    let spacing = Vec2::new(50.0, 50.0);
    rsx! {
        <KayakAppBundle>
            <WindowContextProviderBundle>
                {
                    // Opacity layers are limited to 5 maximum at a time for a given camera.
                    let max_windows = 5;
                    let mut alpha = 0.1;
                    for i in 1..(max_windows + 1) {
                        constructor! {
                            <WindowBundle
                                window={KWindow {
                                    title: format!("Window {} - opacity {}%", i, (alpha * 100.0)),
                                    draggable: true,
                                    initial_position: spacing * i as f32,
                                    size: Vec2::new(300.0, 250.0),
                                    ..KWindow::default()
                                }}
                                styles={KStyle {
                                    // Any time opacity is less than 1.0 a new opacity layer is created.
                                    // WARNING! opacity layers are expensive operations as they are essentially render targets.
                                    // Please use them sparingly ideally only for animations where you have a brief window of opacity.
                                    opacity: alpha.into(),
                                    ..Default::default()
                                }}
                            />
                        }
                        alpha = i as f32 / (max_windows - 1) as f32;
                    }
                }
            </WindowContextProviderBundle>
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugins((KayakContextPlugin, KayakWidgets))
        .add_systems(Startup, startup)
        .run()
}
