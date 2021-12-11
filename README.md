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
- Input events
- Fast and accurate layouts using morphorm: https://github.com/geom3trik/morphorm
- A bunch of default components check out `kayak_components`!
- Style system built to kind of mimic CSS styles.
- Image and Nine patch rendering.

## Bevy Renderer Features
- Image and NinePatch renderer
- Fast MSDF Font renderer
- Quad renderer with rounded corners.
- Custom UI node to ensure UI renders on top of 3D and 2D entities.
- Fully integrated into bevy to capture input events, use bevy assets(images, etc).

## Example Screenshot
<img src="images/screen1.png" alt="Kayak UI" width="600" />

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
pub fn MyCustomWidget(children: Children, styles: Option<Style>) {
    rsx! {
        <>
            {children}
        </>
    }
}
```

## Widget State
```rust
#[widget]
fn Counter(context: &mut KayakContext) {
    let count = {
        let x = context.create_state(0i32).unwrap();
        *x
    };

    let id = self.get_id();
    let on_event = OnEvent::new(move |context, event| match event.event_type {
        EventType::Click => {
            context.set_current_id(id);
            context.set_state(count + 1);
        }
        _ => {}
    });

    rsx! {
        <>
            <Window position={(50.0, 50.0)} size={(300.0, 300.0)} title={"Counter Example".to_string()}>
                <Text size={32.0} content={format!("Current Count: {}", count).to_string()} />
                <Button on_event={Some(on_event)}>
                    <Text size={24.0} content={"Count!".to_string()} />
                </Button>
            </Window>
        </>
    }
}
```