# Creating Props

Before we can jump into creating full-blown widgets, we first need to understand how to create the props that power a widget. Props are defined as any struct that implements the `WidgetProps` trait. This trait is used internally to get and set all relevant data tied to your widget. Let's look at an example.

## Manual Implementation

Say we have a widget that needs to display player information: health, score, mana, etc. So let's create a struct to store all that data.

```rust,noplayground
#[derive(Debug, Default, Clone, PartialEq)] // <- Required traits
pub struct PlayerHudProps {
  pub health: i32,
  pub score: i32,
  pub mana: Option<u8>,
}
```

Before we go further, let's understand what this struct is telling us.

Firstly, all fields are `pub`. This means they can be set by other widgets outside the module this struct is defined in. 

Secondly, `mana` has the type `Option<u8>`, while the others are plain `i32` types. What does this mean? Well, when a widget is defined, it's given the default of the props struct. In our case, this would be `PlayerHudProps::default()`. So when our widget receives these props it will essentially be given the following:

```rust,noplayground
PlayerHudProps {
  health: 0i32,
  score: 0i32,
  mana: None,
}
```

Keep that in mind. If you want optional props, you should probably use `Option`, otherwise you'll be given the default values.

### Common Props

You'll probably want to include some [common props](./common_props.md) at some point. So let's add one. Maybe our widget should also receive input events. In order to do that, we'll need to add a field to store an event handler.

```rust,noplayground
# use kayak_ui::core::OnEvent;
#[derive(Debug, Default, Clone, PartialEq)] // <- Required traits
pub struct PlayerHudProps {
  pub health: i32,
  pub score: i32,
  pub mana: Option<u8>,
  pub on_event: Option<OnEvent>
}
```

> Common props do not need to follow naming conventions, but it is highly recommended. For example, we could have called `on_event` something like `handler`.

But how does Kayak know that this is a common prop? To specify that, we now need to get into implementing `WidgetProps`.

### Implementing

Okay now that we have our struct defined, we need to actually implement `WidgetProps`.

```rust
# use kayak_ui::core::{Children, OnEvent, styles::Style, WidgetProps};
impl WidgetProps for PlayerHudProps {
    fn get_children(&self) -> Option<Children> { None }
    fn set_children(&mut self, children: Option<Children>) {}
    fn get_styles(&self) -> Option<Style> { None }
    fn get_on_event(&self) -> Option<OnEvent> {
      // Return our event handler prop so Kayak can actually use it
      self.on_event.clone()
    }
    fn get_focusable(&self) -> Option<bool> { None }
}
```

That's it! You can now use this struct as props for your widget. You'll notice that the only things we have to define are for common props. Kayak doesn't require you give it any more information about your props other than these few things.

## Deriving `WidgetProps`

A simpler method of implementing `WidgetProps` comes from using the derive macro. This will automatically implement `WidgetProps` for you, which can be pretty useful if you're not needing to specify custom logic in the implementation (such as basing the return value on another prop).

```rust
# use kayak_ui::core::{OnEvent, WidgetProps};
#[derive(WidgetProps, Debug, Default, Clone, PartialEq)]
pub struct PlayerHudProps {
  pub health: i32,
  pub score: i32,
  pub mana: Option<u8>,
  #[prop_field(OnEvent)]
  pub on_event: Option<OnEvent>
}
```

The code above automatically generates the implementation code we just looked at. Since we didn't have any custom logic, this is the simplest way of defining props.

Common props still need to be marked, though. In order to do that we use the `#[prop_field]` helper attribute. This allows us to mark our common props quite succinctly. You can see we added the `#[prop_field(OnEvent)]` attribute to our `on_event` field.

> For a full list of common props and their respective `#[prop_field]` identifiers, check out [this](./common_props.md) page.
