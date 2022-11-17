# Chapter 1 - Installing and hello world!
Kayak UI is quite easy to setup! First make sure you add it to your cargo.toml file in your project. 

Because a crate has yet to be released this currently this looks like:
```toml
kayak_ui = { git = "https://github.com/StarArawn/kayak_ui/", rev = "9b212e230a5325a3ac6897390ded0bc358eebc80"}
```

Once you've added Kayak UI to your bevy project you can now start to use it! In order for you to copy and run this in your own project don't forget to move the `roboto.kayak_font` and the `roboto.png` files to your asset folder. Optionally you can also generate your own font! See: [Chapter 5 - Fonts](./chapter_5.md)

Hello World Example:
```rust
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

    // The rsx! macro expects a parent_id, a widget_context from the user.
    // It also expects `Commands` from bevy.
    // This can be a little weird at first. 
    // See the rsx! docs for more info!
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
    
    commands.spawn(UICameraBundle::new(widget_context));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
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
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));
    commands.spawn(UICameraBundle::new());
    let mut widget_context = KayakRootContext::new();

    let app_entity = widget_context.spawn_widget(&mut commands, None);
    // Create default app bundle
    let mut app_bundle = KayakAppBundle {
        ..Default::default()
    };

    // Create app's children
    let mut children = KChildren::new();

    // Create the text child
    let text_entity = widget_context.spawn_widget(&mut commands, Some(app_entity));
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
    
 commands.spawn(UICameraBundle::new(widget_context));
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
