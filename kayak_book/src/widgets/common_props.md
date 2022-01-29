# Common Props

You may have seen some props like `Children` and `Option<Style>` in this book or in one of
the [examples](https://github.com/StarArawn/kayak_ui/tree/main/examples). These are examples of some of Kayak UI's *
common props*. Such props allow Kayak UI to work seamlessly with your custom widgets, providing all the functionality it
has to offer.

Functional widgets are generated with these props by default. Manually implemented widgets do not necessarily need to
include all of these props, but if they do, it's recommended to follow the naming convention set by the functional
widgets.

## Children

*Functional Signature: `children: Children`*

This is the *most* common of the common props, as well as the most unique. It is used to pass the optional child widget(
s) that should be attached to this widget.

The type `Children` is actually an alias:

```rust,noplayground
pub type Children = Option<
	Arc<dyn Fn(WidgetTree, Option<Index>, &mut KayakContext) + Send + Sync>
>
```

### Usage

This prop is unique in that it is rarely used as a standard prop; rather, it is usually defined with its own syntax.
You've probably seen it before:

```rust,noplayground
# #[widget]
# fn Parent(children: Children) {
#	rsx! { <>{children}</> }
# }
#
# #[widget]
# fn Child() {
#	// ...
# }
#
# #[widget]
# fn MyWidget() {
  rsx! {
    <Parent>
      <Child />
      <Child />
      <Child />
    </Parent>
  }
# }
```

By placing our `Child` widgets between the opening and closing tags of our `Parent` widget, we're actually passing all
three of them into the `children` prop.

## Styles

*Functional Signature: `styles: Option<Style>`*

As its name suggests, this prop is used to pass along styles.

### Usage

```rust,noplayground
# #[widget]
# fn MyWidget() {
  let text_styles = Style {
    color: StyleProp::Value(Color::new(1.0, 0.0, 0.0, 1.0)),
    ..Default::default()
  };

  rsx! {
    <Text content={"Error!".to_string()} size={18.0} styles={Some(text_styles)} />
  }
# }
```

> Be sure to wrap your styles in `Some(...)`!

## OnEvent

*Functional Signature: `on_event: Option<OnEvent>`*

This prop allows an event listener to be attached to a widget.

### Usage

```rust,noplayground
# #[widget]
# fn MyWidget() {
  let event_handler = OnEvent::new(move |_, event| {
    match event.event_type {
      EventType::Click(..) => println!("Clicked!"),
      _ => {}
  	}
  });

  rsx! {
    <Button on_event={Some(event_handler)}>
      <Text content={"Click Me!".to_string()} size={16.0} />
    </Button>
  }
# }
```

> Be sure to wrap your handler in `Some(...)`!

## Focusable

*Functional Signature: `focusable: Option<bool>`*

This prop allows you to set the focusability of a widget. For more details on focusability check out the [Focus Events](../events/focus.md) section.

### Usage

```rust,noplayground
# #[widget]
# fn MyWidget() {
  rsx! {
    <Text focusable={Some(true)} content={"I'm focusable!".to_string()} size={18.0} />
  }
# }
```

> Be sure to wrap your bool in `Some(...)`!