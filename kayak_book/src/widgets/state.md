# State

One of the great things about Kayak UI is that it's *reactive*. This is different from immediate mode and event-driven GUIs in that updating a widget's state *automatically* re-renders the widget— without having to rebuild the widget tree.

But what is state? And what makes it different from props or other data?

## Why State?

Pretend we want to create a `Health` widget that displays a player's health points (HP). It takes the player's HP as a prop:

```rust,noplayground
#[widget]
fn Health(hp: i32) {
  // ...
}
```

Every time that prop changes, you can be sure the `Health` widget itself will also update. Is a prop state then? Let's come back to that in just a bit (see [this](#props-vs-state) section if you're impatient).

Now say you want to allow the user to toggle between actual value and percentage when clicked. This one seems pretty straightforward, right?

```rust,noplayground
#[widget]
fn Health(hp: i32) {
	// DON'T DO THIS
  let mut is_percent = false;
  let event_handler = OnEvent::new(move |_, event| match event.event_type {
		EventType::Click => {
			is_percent = !is_percent;
			println!("Percentage Mode Enabled: {}", is_percent);
		},
		_ => {}
  });
  // ...
}
```

If you run this, you'll find that the widget never toggles. Why is that? The output shows that `is_percent` is indeed toggling, but the widget is never updated.

Something that might help to understand this is to consider our widget as a black box: what happens inside is unknown to Kayak. If we mutate `is_percent`, how will Kayak know to re-render the widget? Not only that, but what are we really mutating? The `is_percent` in the handler is actually a copy of the `is_percent` outside of it due to how closures work in Rust. So even if we could tell Kayak to re-render, the outer value will have been lost when we come back around to re-render.

To break out of this black box, tell Kayak that we need to re-render, and maintain the value across renders, we can look to *state*.

## Creating State

State is a special container for our data that allows it to be maintained across renders and to notify Kayak when a re-render is necessary.

To create state we have two methods (currently). The first is to create the state directly using `context`:

```rust,noplayground
# #[widget]
# fn MyWidget() {
  let state = context.create_state(0i32).unwrap();
# }
```

When the widget is first created, it will register this state with the `KayakContext`, with the given data used as the initial value. This returns an `Option<Binding<T>>` that can be used to `get` the current value or `set` the next value:

```rust,noplayground
// Required for getting and setting
use kayak_ui::core::{Bound, MutableBound};

# #[widget]
# fn MyWidget() {
let state = context.create_state(0i32).unwrap();
let current_value = state.get();
state.set(123);
# }
```

When setting a value, the binding will compare the current value with the given one. If they are not equal, then it will notify Kayak that a re-render should take place (this is why the above code doesn't result in an infinite render loop).

So now we can update our `Health` widget to look something like this:

```rust,noplayground
use kayak_ui::core::{Bound, MutableBound};
#[widget]
pub fn Health(hp: i32) {
  let is_percent = context.create_state(false).unwrap();
  let event_handler = OnEvent::new(move |_, event| match event.event_type {
		EventType::Click => {
			is_percent.set(!is_percent.get());
			println!("Percentage Mode Enabled: {}", is_percent.get());
		},
		_ => {}
  });
  // ...
}
```

Creating state via `context` can be a bit cumbersome, however. You need to unwrap, call get and set everywhere, and add the required use statements in order to get/set the value. If you're okay with hiding a bit of the most common logic, you can instead use the second method for creating state: the `use_state!` proc macro.

The `use_state!` proc macro allows you to get the state much easier in most cases. It returns a tuple containing: the current value, a closure that allows you to set the value, and the state object itself. This results in a much simpler usage:

```rust,noplayground
# #[widget]
# fn MyWidget() {
  let (value, set_value, state) = use_state!(0i32);
  
  // Or, if you don't need the raw state:
  let (value, set_value, ..) = use_state!(0i32);
# }
```

> This assumes access to the widget's `context` (which should not be renamed or reassigned)

Again, we can update our `Health` widget:

```rust,noplayground
#[widget]
fn Health(hp: i32) {
  let (is_percent, set_is_percent, ..) = use_state!(false);
  let event_handler = OnEvent::new(move |_, event| match event.event_type {
		EventType::Click => {
			set_is_percent(!is_percent);
			println!("Percentage Mode Enabled: {}", is_percent);
		},
		_ => {}
  });
  // ...
}
```

### Context vs Macro

So which method should you use? It's really up to you and your needs. The proc macro is great for simple states, especially ones involving primitive types. Creating the state manually through context, on the other hand, gives you a bit more control (especially when it comes to specifying the type using the turbofish syntax).

## Props vs State

Going back to our `Health` widget, updating the prop seems to cause our widget to re-render. So are props state?

The simple answer: no, they're not state.

Recall that the only way for us to set change values is by using state. This might seem obvious now that we've discussed how state works, but in order for that prop to have changed in the first place, it had to have been a state on the parent widget.

```rust,noplayground
#[widget]
fn Health(hp: i32) {
  // ...
}

#[widget]
fn PlayerHud() {
	let (hp, set_hp, ..) = use_state!(20i32);
	
	// ...
	
	rsx! {
		<Health hp={hp} />
	}
}
```

And if this is the case, then when `hp` changes, it causes `PlayerHud` to re-render, which results in `Health` *also* being re-rendered. So `Health` re-rendering is not the prop's doing but the parent's.

> As a side-note, `hp` does not need to specifically be a state on the immediate parent. It could be passed down from the parent's parent or taken as a field from another state. The point is that at some point up the tree, `hp` had to have come from some sort of state in order to have the ability to change.

## Conditional States

One important detail about states is how they're managed internally. States are created by `KayakContext` and use two things to track which data belongs to which state: type and order.

When creating a state,`KayakContext` first checks if a state with the same type already exists for that widget. If it doesn't, great! If it does, it needs another method for differentiating between them. The differentiator is the order in which they're created.

Take this code:

```rust,noplayground
# #[widget]
# fn MyWidget() {
  let state_1 = context.create_state(0).unwrap();
  let state_2 = context.create_state(false).unwrap();
  let state_3 = context.create_state(123).unwrap();
# }
```

To `KayakContext` this looks something like:

1. **State 1** - Create a state with value `0` of type `i32` and order `1`
2. **State 2** - Create a state with value `false` of type `bool` and order `1`
3. **State 3** - Create a state with value `123` of type `i32` and order `2` (since we already have a state of type `i32`)

Why is it important to know about this? Well, consider the following:

```rust,noplayground
# #[widget]
# fn MyWidget(some_conditional: bool) {
  if some_conditional {
    let state_1 = context.create_state(0).unwrap();
    // ...
  }
  let state_2 = context.create_state(false).unwrap();
  let state_3 = context.create_state(123).unwrap();
# }
```

When `some_conditional` is true, we follow the same steps as before. But if, on the next render,  `some_conditional` becomes false, then we end up with:

1. **State 1** - Skipped
2. **State 2** - Get state of type `bool` and order `1` → `false`
3. **State 3** - Get state of type `i32` and order `1` → `0` (Whoops! That belongs to State 1)

Because we skipped the reading of State 1, its value ended up being read by State 3, since that became the first state of type `i32`. This can result in strange behavior and is therefore *strongly* cautioned against.