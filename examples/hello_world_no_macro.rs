use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};
fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    let camera_entity = commands
        .spawn(Camera2dBundle::default())
        .insert(CameraUIKayak)
        .id();

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));
    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);

    let app_entity = widget_context.spawn_widget(&mut commands, None, None);
    // Create default app bundle
    let mut app_bundle = KayakAppBundle {
        ..Default::default()
    };

    // Create app's children
    let mut children = KChildren::new();

    // Create the text child
    let text_entity = widget_context.spawn_widget(&mut commands, None, None);
    commands.entity(text_entity).insert(TextWidgetBundle {
        text: TextProps {
            content: "Hello World".into(),
            ..Default::default()
        },
        ..Default::default()
    });
    // Add the text as a child of the App Widget.
    children.add(text_entity);

    // Finalize app bundle and add to entity.
    app_bundle.children = children;
    commands.entity(app_entity).insert(app_bundle);

    // Add app widget to context.
    widget_context.add_widget(None, app_entity);

    // Add widget context as resource.

    commands.spawn((widget_context, EventDispatcher::default()));
}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((KayakContextPlugin, KayakWidgets))
        .add_systems(Startup, startup)
        .run()
}
