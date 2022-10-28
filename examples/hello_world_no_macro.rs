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

    let pre_existing_app_entity = widget_context.get_child_at(None);
    let app_entity = if let Some(entity) = pre_existing_app_entity {
        commands.get_or_spawn(entity).id()
    } else {
        commands.spawn_empty().id()
    };

    // Create default app bundle
    let mut app_bundle = KayakAppBundle {
        ..Default::default()
    };

    // Create app's children
    let mut children = KChildren::new();

    // Create the text child
    let pre_existing_text_entity = widget_context.get_child_at(Some(app_entity));
    let text_entity = if let Some(entity) = pre_existing_text_entity {
        commands.get_or_spawn(entity).id()
    } else {
        commands.spawn_empty().id()
    };
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
    commands.insert_resource(widget_context);
}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
