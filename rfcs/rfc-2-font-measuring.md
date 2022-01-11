## Background
I would like to measure text to make sure the text widget is the correct size. This is important to ensure we calculate layouts correctly and is useful for text scrolling as well.

## Problem
Currently we don't have a way of measuring text(until render). This causes issues because the layout that is calculated doesn't know how big the text widget is and thus it defaults to the size of its parent. This may or may not be correct however. Additionally we don't have a default font which makes things more difficult. We should be able to show the default font as the other fonts are loading in. 

## Solution
I propose two different solutions for the issues above. 

1. KayakContext keeps track of all fonts that are loaded. This allows us to use the available measuring to properly size the Text widget.
2. bevy_kayak_ui uses include_bytes to load in a default font(probably Roboto).

For the first solution an example of how that would look is something like:
```rust
fonts: HashMap<u16, KayakFont>,
```

For the second issue an example would look like:
```rust
let image = Image::new(
    size,
    dimension: TextureDimension::D2,
    data: include_bytes!("./assets/roboto.png"),
    format: TextureFormat::Rgba8Unorm,
);
let atlas_handle = images_assets.add(image);

let mut font = KayakFont::new(
    Sdf::from_bytes(include_bytes!("./assets/roboto.kayak_font")),
    atlas_handle,
);
```

## Additional issues:
The text widget will need to be re-rendered when it's font is loaded in. I suggest quite a simple way of doing this:
```rust
fonts: Binding<HashMap<u16, KayakFont>>,
```

When we add a new font to the hashmap we need to call `set` on the binding. We can use the `notify` function in the text widget to properly notify the widget of re-renders. This has the added downside of re-rendering all text whenever a font loads in.

### TL;DR
1. Add default font in bevy_kayak_ui
2. Add hash map wrapped in binding that stores kayak fonts.
3. Implement the measuring in the default text widget.

## Alternatives
Another solution is to use an additional data structure:
```rust
fonts: HashMap<u16, KayakFont>,
font_bindings: HashMap<u16, Binding<u16>>,
```

Unresolved issues with this method:
1. How do we initially populate the binding? For example if font id 1 isn't loaded in yet, but a Text widget asks for 2? Do we create it then?

## Feedback
All feedback is welcome!