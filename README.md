<p align="center">
    <img src="images/kayak.svg" alt="Kayak UI" width="600" />
</p>
<br/>

<h1>
    <p align="center">
    Kayak UI
    <p>
</h1>

## What is Kayak UI?
Kayak UI is a declarative UI that can be used to make user interfaces in Rust primarily targeting games. It's free an open-source!

## WARNING
Kayak UI is in the very early stages of development. Important features are missing and documentation is non-existent. There are a few weird oddities because of how the rsx proc macro works, but these could be fixed in the future. Currently kayak is built to be used inside of bevy and as such the existing renderer is built with that in mind, however Kayak UI is render agnostic and could be rendered using any modern rendering API. 

## Features
- Easy to use declarative syntax using a custom proc macro
- Basic widget and global state management
- Input events (Mouse, Keyboard, Char)
- Fast and accurate layouts using morphorm: https://github.com/geom3trik/morphorm
- A few default widgets check out [kayak_widgets](./kayak_widgets)!
- Style system built to kind of mimic CSS styles.
- Image and Nine patch rendering.
- Vec widgets see vec_widget example!
- Removal of widgets.

## Bevy Renderer Features
- Image and NinePatch renderer
- Fast MSDF Font renderer
- Quad renderer with rounded corners.
- Custom UI node to ensure UI renders on top of 3D and 2D entities.
- Fully integrated into bevy to capture input events, use bevy assets(images, etc).
- Dpi Scaling

## Missing features
- Widget prop diffing see issue: https://github.com/StarArawn/kayak_ui/issues/1
- More default widgets.
- More events

## Example Screenshot
<img src="images/screen1.png" alt="Kayak UI" width="600" />

## Usage
Because bevy's new renderer is not released yet there is no crate on crates.io yet. For now please use the following:

```rust
kayak_ui = { git="https://github.com/StarArawn/kayak_ui", rev="{INSERT_COMMIT_SHA_HERE}" }
kayak_widgets = { git="https://github.com/StarArawn/kayak_ui", rev="{INSERT_COMMIT_SHA_HERE}" }
bevy = { git="https://github.com/bevyengine/bevy", rev="{INSERT_COMMIT_SHA_HERE}" }
```

It's also worth mentioning that you will need to use the same bevy revision that this library uses which is currently: `081350916c39fc1c7c9d659ccb6d86413832b465`.

This is temporary and will change when bevy 0.6 is released.

## Declarative
Kayak UI makes it painless to build out complex UI's using custom or pre-built widgets. Custom widgets are layed out in a XML like syntax that allows you to more easily visualize the widget tree. Here's an example of that syntax:
```rust
rsx! {
    <App>
        <Button styles={Some(play_button_styles)}>
            <Text
                size={30.0}
                content={"Hello World!".to_string()}
            />
        </Button>
    </App>
}
```

You can easily declare your own custom widgets:
```rust
#[widget]
pub fn MyCustomWidget(children: Children) {
    rsx! {
        <>
            {children}
        </>
    }
}
```

## Widget State

Widget's can create their own state and will re-render when that state changes.
```rust
#[widget]
fn Counter(context: &mut KayakContext) {
    let (count, set_count, ..) = use_state!(0i32);
    let on_event = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click => set_count(count + 1),
        _ => {}
    });

    let count_text = format!("Current Count: {}", count);
    rsx! {
        <>
            <Window position={(50.0, 50.0)} size={(300.0, 300.0)} title={"Counter Example".to_string()}>
                <Text size={32.0} content={count_text} />
                <Button on_event={Some(on_event)}>
                    <Text size={24.0} content={"Count!".to_string()} />
                </Button>
            </Window>
        </>
    }
}
```

Widget's can also access global state and when the global state is bound to the widget it will auto re-render:
```rust
#[widget]
fn Counter(context: &mut KayakContext) {
    let global_count = {
        if let Ok(world) = context.get_global_state::<World>() {
            if let Some(global_count) = world.get_resource::<Binding<GlobalCount>>() {
                global_count.clone()
            } else {
                return;
            }
        } else {
            return;
        }
    };

    // Binds the global state to the widget.
    // When `global_count.set()` is called the Counter widget will auto re-render.
    context.bind(&global_count);
    let global_count = global_count.get().0;
    
    rsx! {
        <>
            <Window position={(50.0, 50.0)} size={(300.0, 300.0)} title={"Counter Example".to_string()}>
                <Text size={32.0} content={format!("Current Count: {}", global_count).to_string()}>{}</Text>
            </Window>
        </>
    }
}

// Example bevy system:
fn count_up(global_count: Res<Binding<GlobalCount>>) {
    global_count.set(GlobalCount(global_count.get().0 + 1));
}
```

## Creating new fonts
The `bevy_kayak_ui` renderer uses MSDF fonts in order to render crisp and accurate fonts at different scales without needing to re-rasterize the font. In order to generate custom fonts you need to use the following tool:
https://github.com/Chlumsky/msdf-atlas-gen

To generate a font run the following command:
```
.\msdf-atlas-gen.exe -font .\font_name.ttf -type msdf -minsize 32 -format png -imageout font_name.png -json font_name.json
```
Where font_name is the name of your font. You can play around with the different parameters that are provided but keep in mind that some of the font stuff is currently hardcoded and might result in graphical glitches if you change the settings too much. You should also try to use a decent size for the `minsize` parameter. The smaller the size the more artifacts will appear in the text.