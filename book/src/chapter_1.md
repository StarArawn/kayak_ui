# Chapter 1 - Installing and hello world!
Kayak UI is quite easy to setup! First make sure you add it to your cargo.toml file in your project. 

```toml
kayak_ui = "0.2"
```

Once you've added Kayak UI to your bevy project you can now start to use it! In order for you to copy and run this in your own project don't forget to move the `roboto.kayak_font` and the `roboto.png` files to your asset folder. Optionally you can also generate your own font! See: [Chapter 5 - Fonts](./chapter_6.md)

Hello World Example:
```rust
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

    font_mapping.set_default(asset_server.load("roboto.kttf"));

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <TextWidgetBundle
                text={TextProps {
                    content: "Hello World".into(),
                    size: 20.0,
                    ..Default::default()
                }}
            />
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}

```

## Wait where is the ECS?
Kayak UI encourages the use of our proc macro called `rsx!`. This proc macro simulates XML like syntax and turns it into bevy commands. This proc macro is completely optional though!

Here is the same hello world example but this time we'll use bevy's ECS directly.

```rust
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

    font_mapping.set_default(asset_server.load("roboto.kttf"));
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
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
```
