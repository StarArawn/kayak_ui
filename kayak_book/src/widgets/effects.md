# Side-Effects

What is a side-effect? In simple terms, a side-effect is something that happens as a result of something else happening. For example, take this simple function:

```rust
fn add(a: i32, b: i32): i32 {
  let sum = a + b;
  return sum;
}
```

It purely takes in an input and gives an output. That's it. No other state mutated within its body. Now add one line of code:

```rust
fn add(a: i32, b: i32): i32 {
  let sum = a + b;
  println!("Sum: {}", sum);
  return sum;
}
```

We just changed our *pure* function to one that contains a side-effect (logging to stdout). Now every time we call `add` we get the side-effect of printing out the sum.

We can do a similar thing with widgets, wherein when a state is changed, a side-effect function may be called.

## Creating a Side-Effect

A side-effect (we'll call them "effects" for short from here on) can be registered to the `KayakContextRef` with a set of state dependencies. When any of the given states are changed, the effect closure will be called. These dependencies are passed by reference into the *dependency array*.

```rust,noplayground
# #[widget]
# fn MyWidget() {
  let (value, set_value, value_state) = use_state!(0);
  let other_state = context.create_state(0).unwrap();
  
  context.create_effect(move || {
    println!("Detected a change");
  }, &[&value_state, &other_state]);
# }
```

### Accessing Values

One complication with these effects (that may be fixed in the future) is that variables shared with the closure are delayed. So printing out the current value for a counter will always print the last value, not the current one. The reason this happens is that the closure is registered upon render, and all captured variables are captured with their current values. When a dependency state changes, it runs the closure before re-rendering, leaving the actual value out of sync.

In order to get the actual current value, you'll need to do two things:

1. Make sure all variables passed into the closure are state objects
2. Use the state object's `get` method to retrieve the value

This ends up looking like this:

```rust,noplayground
// Make sure include this for getting and setting state
use kayak_core::{Bound, MutableBound};
# #[widget]
# fn MyWidget() {
  let (value, set_value, value_state) = use_state!(0);
  
  // Clone the state since we need to use it again for the dependency array
  let state = value_state.clone();
  context.create_effect(move || {
    println!("Old Value: {} | New Value: {}", value, state.get());
  }, &[&value_state]);
# }
```

### Empty Dependency Array

An empty dependency array will cause the effect closure to only run once, upon first render. This is particularly useful for setup/initialization logic.

```rust,noplayground
# #[widget]
# fn MyWidget() {
  context.create_effect(move || {
    println!("I run once!");
  }, &[]);
# }
```

## `use_effect!`

To help make effects easier to register and use, you may also use the `use_effect!` proc macro. Using the macro we can  replace our existing code with this:

```rust,noplayground
# use kayak_core::{Bound, MutableBound};
# #[widget]
# fn MyWidget() {
  let (value, set_value, value_state) = use_state!(0);
  
  use_effect!(move || {
    println!("Old Value: {} | New Value: {}", value, value_state.get());
  }, [value_state]);
# }
```

While just being a bit cleaner to write (notice how the dependency array doesn't need all those `&` operators), it also performs some interesting maneuvering under the hood.

You may have noticed that we're using the `value_state` variable in both the closure and the dependency array. This shouldn't be possible, right?

Well the macro takes the common pattern of cloning your state for use in the closure, and does it for you. The macro actually generates something along the lines of:

```rust,noplayground
# #[widget]
# fn MyWidget() {
  let (value, set_value, value_state) = use_state!(0);
  
  {
    // Automatically includes these use statements for you
    use kayak_core::{Bound, MutableBound};

    // Clones the dependnecies
    let value_state_dependency_clone = value_state.clone();
    
    // Registers the effect
    context.create_effect(move || {
      println!("Old Value: {} | New Value: {}", value, value_state.get());
    }, &[&value_state_dependency_clone]);
  }
# }
```

> Ensure that `context` point to the `KayakContextRef` (i.e. it should not be reassigned). Also try not to use variables in your closure with the same names as the generated variables, which follow the pattern `<State Name>_dependency_clone`. Hopefully that won't be an issue...

### Context vs Macro

Much like the [`use_state!` macro](./state.md#use_state), whether you use the macro or manually create it with `context`, doesn't really matter. The decision is mostly up to personal preference.

## Conditional Effects

Effects work much like state in that the order in which they are registered matters. This means you should almost always avoid conditionally registering an effect as it can lead to unintended behavior. For example, you should not do this:

```rust,noplayground
# #[widget]
# fn MyWidget(some_conditional: bool) {
#   let some_state = context.create_state(0).unwrap();
  if some_conditional {
    use_effect(|| {
      println!("I may or may not run");
    }, []);
  }
  
  use_effect(|| {
    // If some_conditional is false, this will print "I may or may not run"
  }, [some_state]);
# }
```

Unlike state, however, effects only consider their order when registering, not type. The dependency array is also not considered.

For more information on what this all means and why it's important, check out the section on [conditional states](./state.md#conditional-states).
