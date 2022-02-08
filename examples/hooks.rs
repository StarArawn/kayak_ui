//! This example file demonstrates a few of the most common "hooks" used in this crate. For Kayak, a hook works much like
//! hooks in React: they hook into the lifecycle of their containing widget allowing deeper control over a widget's internal
//! logic.
//!
//! By convention, the macro "hooks" all start with the prefix `use_` (e.g., `use_state`). Another important thing to keep
//! in mind with these hooks are that they are just macros. They internally setup a lot of boilerplate code for you so that
//! you don't have to do it all manually. This means, though, that they may add variables or rely on external ones (many
//! hooks rely on the existence of a `KayakContext` instance named `context`, which is automatically inserted for every
//! widget, but could unintentionally be overwritten by a user-defined `context` variable). So be mindful of this when adding
//! these hooks to your widget— though issues regarding this should be fairly rare.
//!
use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    core::{render, rsx, use_effect, use_state, widget, EventType, Index, OnEvent},
    widgets::{App, Button, Text, Window},
};

/// A simple widget that tracks how many times a button is clicked using simple state data
#[widget]
fn StateCounter(props: StateCounterProps) {
    // On its own, a widget can't track anything, since every value will just be reset when the widget is re-rendered.
    // To get around this, and keep track of a value, we have to use states. States are values that are kept across renders.
    // Additionally, anytime a state is updated with a new value, it causes the containing widget to re-render, making it
    // useful for updating part of the UI with its value.
    // To create a state, we can use the `use_state` macro. This creates a state with a given initial value, returning
    // a tuple of its currently stored value and a closure for setting the stored value.

    // Here, we create a state with an initial value of 0. Right now the value of `count` is 0. If we call `set_count(10)`,
    // then the new value of `count` will be 10.
    let (count, set_count, ..) = use_state!(0);

    // We can create an event callback that uodates the state using the state variables defined above.
    // Keep the borrow checker in mind! We can pass both `count` and `set_count` to this closure because they
    // both implement `Copy`. For other types, you may have to clone the state to pass it into a closure like this.
    // (You can also clone the setter as well if you need to use it in multiple places.)
    let on_event = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click(..) => set_count(count + 1),
        _ => {}
    });

    rsx! {
        <Window position={(50.0, 50.0)} size={(300.0, 150.0)} title={"State Example".to_string()}>
            <Text size={16.0} content={format!("Current Count: {}", count)} />
             <Button on_event={Some(on_event)}>
                <Text line_height={Some(40.0)} size={24.0} content={"Count!".to_string()} />
            </Button>
        </Window>
    }
}

/// Another widget that tracks how many times a button is clicked using side-effects
#[widget]
fn EffectCounter() {
    // In this widget, we're going to implement another counter, but this time using side-effects.
    // To put it very simply, a side-effect is when something happens in response to something else happening.
    // In our case, we want to create a side-effect that updates a counter when another state is updated.

    // In order to create this side-effect, we need access to the raw state binding. This is easily done by using
    // the third field in the tuple returned from the `use_state` macro.
    let (count, set_count, raw_count) = use_state!(0);
    let on_event = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click(..) => set_count(count + 1),
        _ => {}
    });

    // This is the state our side-effect will update in response to changes on `raw_count`.
    let (effect_count, set_effect_count, ..) = use_state!(0);

    // This hook defines a side-effect that calls a function only when one of its dependencies is updated.
    // They will also always run upon first render (i.e., when the widget is first added to the layout).
    use_effect!(
        move || {
            // Update the `effect_count` state with the current `raw_count` value, multiplied by 2.
            // Notice that we use `raw_count.get()` instead of `count`. This is because the closure is only defined once.
            // This means that `count` will always be stuck at 0, as far as this hook is concerned. The solution is to
            // use the `get` method on the raw state binding instead, to get the actual value.
            set_effect_count(raw_count.get() * 2);
        },
        // In order to call this side-effect closure whenever `raw_count` updates, we need to pass it in as a dependency.
        // Don't worry about the borrow checker here, `raw_count` is automatically cloned internally, so you don't need
        // to do that yourself.
        [raw_count] // IMPORTANT:
                    // If a side-effect updates some other state, make sure you do not pass that state in as a dependency unless you have
                    // some checks in place to prevent an infinite loop!
    );

    // Passing an empty dependency array causes the callback to only run a single time: when the widget is first rendered.
    use_effect!(
        || {
            println!("First!");
        },
        []
    );

    // Additionally, order matters with these side-effects. They will be ran in the order they are defined.
    use_effect!(
        || {
            println!("Second!");
        },
        []
    );

    rsx! {
        <Window position={(50.0, 225.0)} size={(300.0, 150.0)} title={"Effect Example".to_string()}>
            <Text size={16.0} content={format!("Actual Count: {}", count)} />
            <Text size={16.0} content={format!("Doubled Count: {}", effect_count)} />
             <Button on_event={Some(on_event)}>
                <Text line_height={Some(40.0)} size={24.0} content={"Count!".to_string()} />
            </Button>
        </Window>
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add("Roboto", asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <StateCounter />
                <EffectCounter />
            </App>
        }
    });

    commands.insert_resource(context);
}

fn main() {
    BevyApp::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("UI Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
}
