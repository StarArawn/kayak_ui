# Macros
## Builders

These are macros related to building widgets, most of which will certainly be used in your own code.

### `#[widget]`

This attribute macro is what allows functions to be generated into valid widgets structs.

```rust,noplayground
#[widget]
fn MyWidget() { /* ... */ }
```

It also accepts a `focusable` parameter for quickly designating a widget as [focusable](./widgets/common_props.md#focusable).

```rust,noplayground
#[widget(focusable)]
fn MyFocusableWidget() { /* ... */ }
```

### `rsx!`

A proc macro that turns RSX syntax into structure constructors[^1] and calls the context to create the widgets.

```rust,noplayground
#[widget]
fn MyWidget() {
  rsx! {
    <Text content={"Hello World".to_string()} size={18} />
  }
}
```

### `constructor!`

A proc macro that turns RSX syntax into structure constructors only.

TODO: Elaborate on this and how it's different from `rsx!`.

You'll often use this with things like `VecTracker`.

```rust,noplayground
#[widget]
fn PlayerNames() {
  rsx! {
    {VecTracker::from(names.iter().map(|name| {
      constructor! {
          <Text content={name.clone()} size={16.0} />
      }
    }))}
  }
}

#[derive(WidgetProps, Debug, Default, Clone, PartialEq)]
struct PlayerNamesProps {
  pub names: Vec<String>
}
```

### `render!`

A top level macro that works the same as RSX but provides some additional context for building the root widget.

```rust,noplayground
let context = BevyContext::new(|context| {
    render! {
        <App>
          <MyWidget />
        </App>
    }
});
#
# #[widget]
# fn MyWidget() {
#   // ... 
# }
```

## Helpers

These macros are mainly meant to make commonly used code easier to write and nicer to look at.

### `use_state!`

This macro is syntactic sugar for:

```rust,noplayground
# #[widget]
# fn MyWidget() {
  let state = context.create_state(0).unwrap();
  
  // Get the underlying value
  let value = state.get();
  
  // Clone the state so we can pass it to closures
  let cloned_state = state.clone();
# }
```

Allowing you to instead write:

```rust,noplayground
# #[widget]
# fn MyWidget() {
  let (value, set_value, state) = use_state!(0);
# }
```

Check out [this](./widgets/state.md#use_state) section for more details.

### `use_effect!`

Like `use_state!`, this macro is syntactic sugar, which can be used to replace code like this:

```rust,noplayground
use kayak_core::{Bound, MutableBound};
# #[widget]
# fn MyWidget() {
  let (value, set_value, value_state) = use_state!(0);
  
  let state = value_state.clone();
  context.create_effect(move || {
    println!("Old Value: {} | New Value: {}", value, state.get());
  }, &[&value_state]);
# }
```

With code like this:

```rust,noplayground
# #[widget]
# fn MyWidget() {
  let (value, set_value, value_state) = use_state!(0);
  
  use_effect!(move || {
    println!("Old Value: {} | New Value: {}", value, value_state.get());
  }, [value_state]);
# }
```

Check out [this](./widgets/effects.md#use_effect) section for more details.



<br />

---

[^1]: TODO: Elaborate on "structure constructors"