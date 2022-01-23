# Getting Started

## Installation 
`kayak_ui` is not available via Rust's official package repository yet. As such it is recommended that you bring in the crate via cargo's git feature:
```toml
[dependencies]
kayak_ui = { git = "https://github.com/StarArawn/kayak_ui", rev="b489c6b64187ab926624604738037726cfb4296b", features = "bevy_renderer" }
```

## Hello World!


First start with a standard bevy app:
```rust
fn main() {
    BevyApp::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Hello World!"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .run();
}
```

To add Kayak UI as a bevy plugin import the module you'll also want to bring in the `Context`, `FontMapping`, and `UICameraBundle`. Make sure to add `BevyKayakUIPlugin` to the bevy app using `add_plugin`! Finally we'll add the widgets `App` and `Text` for this example and a macro for creating our JSX like tree.

```rust
use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    core::{render, Index},
    widgets::{App, Text},
};
...

app.add_plugin(BevyKayakUIPlugin)
```

Next create a startup system. We'll use that to create our Kayak UI context, load in fonts, and build out our widget tree!
```rust
fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // Font mapping maps bevy kayak fonts to kayak.
    // We need this because we use bevy asset system to load things in!
    mut font_mapping: ResMut<FontMapping>,
) {
    // You can load in any kayak fonts and they are key'd by a String.
    font_mapping.add("Roboto", asset_server.load("roboto.kayak_font"));

    // Add the bevy kayak ui camera:
    commands.spawn_bundle(UICameraBundle::new());

    // Now we create the kayak context which is wrapped in a BevyContext resource.
    let context = BevyContext::new(|context| {
        // Using render we can create our widgets. Here we have an `App` which is recommended as the 
        // base widget that should be used. With the bevy feature it automatically sizes everything to
        // scale with the bevy window.
        render! {
            <App>
                <Text size={26.0} content={"Hello World".to_string()} />
            </App>
        }
    });

    // Finally insert the context into bevy as a resource.
    commands.insert_resource(context);
}

// Don't forget to add the startup system to bevy!
```

To see a complete example check out the [hello_world](../examples/hello_world.rs) example!