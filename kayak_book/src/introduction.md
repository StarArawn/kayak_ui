<p align="center">
    <img src="img/kayak.svg" alt="Kayak UI" width="600" />
</p>
<br/>

# Introduction 
**Kayak UI** is a declarative and reactive UI that can be used to make user interfaces in Rust primarily targeting games. It supports an optional custom JSX like syntax for building out widgets. If you've used React before in the past you'll feel right at home with some of the concepts here.

Initially when I wrote this library I thought I had a grasp about how complex a GUI library can be, but this has been a learning experience into just how complex it really is. I'm hoping I can help teach some of these things and make it easier for people to write good UI/GUI for their games!

Currently Kayak UI is renderer agnostic, however it is tightly integrated into the [Bevy Game Engine](https://bevyengine.org/). As such all examples in this book use Bevy.

## Reactive
What does it mean to be reactive? In Kayak UI we have state(pieces of data) and the ability to bind widgets to that state. Some state is local to a specific widget and some of it is global. If you are using Kayak in bevy the bevy ECS world is essentially your global state. When state is bound to a widget if that data changes the widget is re-rendered. A render only occurs when state changes. This has benefits over traditional gamedev GUI's in the past that are typically immediate or event driven. 

## Declarative
A declarative GUI is a way of defining preassembled widgets that are reusable building blocks to more easily build out your UI. Kinda like lego bricks. Here's an example:
```rust
    <Button styles={Some(play_button_styles)}>
        <Text
            size={30.0}
            content={"Hello World!".to_string()}
        />
    </Button>
```

It's easy to view and change what gets rendered and is displayed to the end user. It's really not much different from HTML. 

## WARNING
Kayak UI is in the very early stages of development. Important features are missing and documentation is non-existent. There are a few weird oddities because of how the rsx proc macro works, but these could be fixed in the future. Currently kayak is built to be used inside of bevy and as such the existing renderer is built with that in mind, however Kayak UI is render agnostic and could be rendered using any modern rendering API.


## Contributing
Kayak UI is free and open source. You can find the source code on [GitHub](https://github.com/StarArawn/kayak_ui), issues can be posted on the [GitHub issue tracker](https://github.com/StarArawn/kayak_ui/issues), with feature requests and questions directed to [Github discussions](https://github.com/StarArawn/kayak_ui/discussions).