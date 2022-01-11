## Background
I would like to measure text to make sure the text widget is the correct size. This is important to ensure we calculate layouts correctly and is useful for text scrolling as well.

## Problem
Currently we don't have a way of measuring text(until render). This causes issues because the layout that is calculated doesn't know how big the text widget is and thus it defaults to the size of its parent. This may or may not be correct however. Additionally we don't have a default font which makes things more difficult. We should be able to show the default font as the other fonts are loading in. 

## Solution
I propose two different solutions for the issues above. 

1. KayakContext keeps track of all fonts that are loaded. This allows us to use the available measuring to properly size the Text widget.
2. bevy_kayak_ui uses include_bytes to load in a default font(probably Roboto).


## How this will look

We'll need a few new types to store the fonts. I would also like to make something more generic and reusable in the future. It'll look something like this:

`AssetStorage`:
```rust
pub struct AssetStorage<T> {
    assets: HashMap<String, Binding<Option<T>>>,
}
```

`KayakContext`:
```rust
pub struct KayakContext {
    ..
    // Stores AssetStorage generically.
    assets: Resources,
}

impl KayakContext {
    // Binding returned here to allow widgets to track loading.
    pub fn get_asset<T>(&self, asset_handle: String) -> &Binding<Option<T>> {
        // Create new asset if asset doesn't exist, and sets it to None.
        ..
    }
 
    // Set's the asset value.
    pub fn set_asset<T>(&mut self, asset_handle: String, asset: T) {
        ..
    }

    // Creates the AssetStorage<T> and stores it in assets
    pub fn initialize_asset<T>(&mut self) {
        ..
    }
}
```

### TL;DR
1. Add default font in bevy_kayak_ui
2. Add new asset storage types and implement logic.
3. Implement the measuring in the default text widget.

## Feedback
All feedback is welcome!