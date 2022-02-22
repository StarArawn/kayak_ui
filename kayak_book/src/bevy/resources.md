# Using Resources

What good is a UI if you can't interact with the rest of the your application? The `bevy_renderer` feature comes with a simple way of accessing the current Bevy `World`.

## Querying

To access this data, we can use the `query_world` method on our `KayakContextRef` (if you're building a functional widget, this is the hidden `context` variable) to access any `SystemParam`. This is akin to defining a typical Bevy system within our widget (although only accepting a single parameter).

```rust
# use kayak_ui::core::{use_state, widget};
# use bevy::prelude::*;
# 
struct GlobalCounter(pub u32);

#[widget]
fn MyWidget() {
  let (inc_amount, set_inc_amount, ..) = use_state!(1u32);
  let value = context.query_world::<Res<GlobalCounter>, _, _>(move |counter| {
    counter.0 += inc_amount;
		counter.0
  });
}
```

Here, the `Res<GlobalCounter>` is the `SystemParam` we're trying to access. The remaining `_` type parameters simply tell the compiler to infer the closure type and the closure's return type, respectively (the second `_`, for example, could be replaced with a concrete `u32` type).

> The `query_world` method only accepts a single parameter, so your "system" should only accept that one. If you need more parameters, consider [creating a custom `SystemParam`](https://github.com/bevyengine/bevy/blob/9a7852db0f22eb41f259a1afbb4926eb73863a10/examples/ecs/system_param.rs#L21-L25) or using a tuple of parameters:
>
> ```rust,ignore
> context.query_world::<(Res<GlobalCounter>, Query<&Transform>), _, _>(/* ... */);
> ```

If you run this code, though, you may notice that our `GlobalCounter` resource increments by 1, but then stops. Why is this? 

The reason this seems to only run once, is that we actually only run this code when the widget is rendered. If we called `set_inc_amount(2)`, for example, that would cause a state update, which would result in a re-render, which would result in the query being called again.

Because this query is called on every re-render (including when a parent updates), you'll need to be careful. It's probably best to add some logic that prevents the resource from being mutated on *every* re-render.

### Binding Resources

You may want your widget to bind to the resource directly, react to changes, and update accordingly. This can be done by wrapping a `Binding<T>` around our resource.

```rust
# use bevy::prelude::*;
# fn setup_count(mut commands: Commands) {
  // ❌ DON'T do this
  commands.insert_resource(GlobalCount(0));
  
  // ✅ DO this
  use kayak_ui::core::bind;
  commands.insert_resource(bind(GlobalCount(0)));
# }
```

By using the `bind` function, we create a `Binding<GlobalCount>` rather than just a `GlobalCount`. This allows us to bind our widget to the resource using `context.bind(...)`:

```rust
# use kayak_ui::core::{Binding, Bound, MutableBound, use_state, widget};
# use bevy::prelude::*;
# 
# struct GlobalCounter(pub u32);
# 
# #[widget]
# fn MyWidget() {
#   let (inc_amount, set_inc_amount, ..) = use_state!(1u32);
  let binding = context.query_world::<Res<Binding<GlobalCounter>>, _, _>(|counter| counter.clone());
  
  // This is what causes the widget to re-render when the resource is changed
  context.bind(&binding);
  
  let value = binding.get().0;
# }
```

And now that we're prepared to react to any changes to our `GlobalCounter` resource, we can actually update it in Bevy-land.

```rust
# use bevy::prelude::*;
# use kayak_ui::core::{Binding, Bound, MutableBound};
// A standard Bevy system that updates our counter
fn update_counter(mut counter: ResMut<Binding<GlobalCounter>>) {
  let current_value = counter.get().0;
  counter.set(GlobalCount(current_value + 1));
}
```

> ⚠️ The `counter` parameter does not actually need to be `mut` nor does it need to be typed as `ResMut`. This is because setting a binding uses interior mutability, thus not needing mutable access. However, Bevy's scheduler won't know this if you simply use `Res`. This can cause data races and result in undefined behavior. **Therefore, it's best to still mark them as mutable using `ResMut`.**

#### Limitations

Using a binding is convenient, but not without its limitations. For one, all bound values need to implement `Clone` and `PartialEq`, since "getting" a value returns a clone of the stored value and setting a value requires comparison with the stored value.

Because of this, you also need to mutate the value in a particular way.

```rust
# use bevy::prelude::*;
# use kayak_ui::core::{Binding, Bound, MutableBound};
// A standard Bevy system that updates our counter
fn update_counter(mut counter: ResMut<Binding<GlobalCounter>>) {
  // You need to update like this:
  let current_value = counter.get().0;
  counter.set(GlobalCount(current_value + 1));
  
  // NOT like this:
  let mut current_value = counter.get().0;
  current_value.0 += 1; // This change won't be retained!
}
```

##### Alternatives

If you can't implement `PartialEq` or `Clone` on your type, and thus can't create a binding for it, one thing you can do is create a secondary type that corresponds to a change in the actual type.

For example, if we have a resource type `Foo` that can't implement `PartialEq`, we can create a `FooChanged` resource that *can*:

```rust
#[derive(Clone)]
struct Foo;

#[derive(Clone, PartialEq)]
struct FooChanged(pub bool);
```

Then we can create a system that toggles `FooChanged` whenever `Foo` is mutated:

```rust
use bevy::prelude::*;
use kayak_ui::core::{Binding, Bound, MutableBound};

# #[derive(Clone)]
# struct Foo;
# 
# #[derive(Clone, PartialEq)]
# struct FooChanged(pub bool);
# 
fn on_foo_change(foo: Res<Foo>, mut foo_changed: ResMut<Binding<FooChanged>>) {
  if(foo.is_changed()) {
    let old_value = foo_changed.get().0;
    foo_changed.set(FooChanged(!old_value));
  }
}
```

With that, we can bind and use the resources in our widget:

```rust
# use kayak_ui::core::{Binding, use_state, widget};
# use bevy::prelude::*;
# 
# #[derive(Clone)]
# struct Foo;
# 
# #[derive(Clone, PartialEq)]
# struct FooChanged(pub bool);
# 
#[widget]
fn MyWidget() {

  // Bind to `FooChanged`
  let foo_changed = context.query_world::<Res<Binding<FooChanged>>, _, _>(|foo_changed| {
    foo_changed.clone()
  });
  context.bind(&foo_changed);
  
  // Use the actual value from `Foo`
  let foo = context.query_world::<Res<Foo>, _, _>(|foo| {
    foo.clone()
  });
}
```

In fact, you may find that a widget needs to know about changes to multiple resources or queries. In that case, such a pattern becomes even more useful. Rename `FooChanged` to `ChangeMyWidget` and toggle its value whenever you need to! 

> Be sure that if you use this pattern, you actually set the value to a *new* value. Otherwise, the binding won't detect a change and the widget won't re-render.
