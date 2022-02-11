# Setup

To get started with Kayak's Bevy integration, all you need to do is install the `kayak_ui` crate as normal:

```toml
kayak_ui = TODO: Add version
```

The `bevy_renderer` feature is enabled by default and allows you to easily access Bevy-related Kayak features under the `kayak_ui::bevy` namespace.

## Usage

With `kayak_ui` installed, and the `bevy_renderer` feature enabled, we're ready to start using Kayak for our Bevy apps.

### Adding the Plugin

To start, we'll need to add the `BevyKayakUIPlugin` to our app.

```rust,noplayground
use bevy::prelude::{App as BevyApp, DefaultPlugins};
use kayak_ui::bevy::BevyKayakUIPlugin;

fn main() {
  BevyApp::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(BevyKayakUIPlugin)
    .run();
}
```

> In code and examples, you'll often see Bevy's `App` imported as `BevyApp`. This is because a very common Kayak widget is also named `App` (TODO: Link to App). However, you could just as well import that one as `KayakApp`, it's up to preference!

This plugin handles *a lot* for us, but the main things are that it:

* Adds systems for automatically updating the UI
* Adds window event listeners
* Sets up the render sub-app

### Creating the Context

In order to actually start adding our UI, we need to add a *context* to contain our UI. This is done by creating and storing a resource called `BevyContext`. We can create a system to do this:

```rust,noplayground
use bevy::prelude::{Commands, UICameraBundle};
use kayak_ui::{
  bevy::BevyContext,
  core::render,
  widgets::App
};

# use kayak_ui::core::widget;
#
# #[widget]
# fn MyWidget() { /* ... */ }
#

fn setup_ui(mut commands: Commands) {
  // Spawn a UI camera to render our UI
  commands.spawn_bundle(UICameraBundle::new());

  // Create the context
  let context = BevyContext::new(|context| {
    render! {
      <App>
        <MyWidget />
      </App>
    }
  });

  // Store the context as a resource
  commands.insert_resource(context);
}
```

You may have noticed the usage of the `render!` macro instead of the standard `rsx!` macro. You can read more about what the `render!` macro does in the [Macros](../macros.md#render) section. Essentially, it sets up the root node of our widget tree in ways `rsx!` does not, so be sure to use the correct one here!

Also, don't forget to insert the context as a **resource**. The systems added by the plugin expect a `BevyContext` resource to exist in order to work properly. This means you can't create multiple at one time and you can't store it as a field on some other resource.

> In the future, we plan to allow for multiple widget trees within a single context. For now, you'll have to use `remove_resource` and `insert_resource` to either remove or overwrite the context, respectively.

### Adding the System

The last thing to do is actually add our system to our app.

```rust,noplayground
# use bevy::prelude::{App as BevyApp, DefaultPlugins};
# use kayak_ui::bevy::BevyKayakUIPlugin;
# 
fn main() {
  BevyApp::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(BevyKayakUIPlugin)
    .add_startup_system(setup_ui)
    .run();
}
```

> Be careful you don't add this using a plain old `add_system` call. Without any checks or run criteria in place, we'll be creating and inserting a new `BevyContext` over and over again each frame!

#### Using State

A common pattern for handling transitioning to and from UIs in Bevy is through states. While we hope to improve this in the future to be more performant and useful, for now you can simply use state transitions to insert/remove your UI dynamically.

```rust
# use bevy::prelude::{App as BevyApp, DefaultPlugins, SystemSet};
# use kayak_ui::bevy::BevyKayakUIPlugin;
# 
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
  InGame,
  MainMenu,
  Options
}

fn destroy_ui() { /* ... */ }
fn create_main_menu() { /* ... */ }
fn create_options_menu() { /* ... */ }

fn main() {
  BevyApp::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(BevyKayakUIPlugin)
  
    // --- Main Menu --- //
    .add_system_set(SystemSet::on_enter(GameState::MainMenu).with_system(create_main_menu))
    .add_system_set(SystemSet::on_exit(GameState::MainMenu).with_system(destroy_ui))
  
    // --- Options Menu --- //
    .add_system_set(SystemSet::on_enter(GameState::Options).with_system(create_options_menu))
    .add_system_set(SystemSet::on_exit(GameState::Options).with_system(destroy_ui))
  
    .run();
}
```

Whenever we transition to a new UI state, we call a system that creates and inserts a new `BevyContext`. When we transition out of this state, we also make sure to destroy that context. Check out the [bevy_state example](https://github.com/StarArawn/kayak_ui/blob/main/examples/bevy_state.rs) to see it in action!

## Supported Versions

Currently, Kayak UI supports the following versions of Bevy:

| Bevy Version | Kayak UI Version  |
| ------------ | ----------------- |
| 0.6          | TODO: Add version |

