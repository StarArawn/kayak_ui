# Using Resources

What good is a UI if you can't interact with the rest of the your application? The `bevy_renderer` feature comes with a simple way of accessing the current Bevy `World`.

## Querying

To access this data, we can use the `query_world` method on our `KayakContextRef` (if you're building a functional widget, this is the hidden `context` variable) to access any `SystemParam`. This is akin to defining a typical Bevy system within our widget (although only accepting a single parameter).

```rust
struct GlobalCounter(pub u32);

#[widget]
fn MyWidget() {
  let (inc_amount, set_inc_amount, ..) = use_state!(1u32);
  let value = context.query_world::<Res<GlobalCounter>>, _, _>(move |counter| {
    counter.0 += inc_amount;
		counter.0
  });
}
```

Here, the `Res<GlobalCounter>` is the `SystemParam` we're trying to access. The remaining `_` type parameters simply tell the compiler to infer the closure type and the closure's return type, respectively (the second `_`, for example, could be replaced with a concrete `u32` type).

> The "system" might be more recognizable if we look at it a different way:
>
> ```rust
> # struct GlobalCounter(pub u32);
> # 
> # #[widget]
> # fn MyWidget() {
> #   let (inc_amount, set_inc_amount, ..) = use_state!(1u32);
>   let query = move |counter: Res<GlobalCounter>| -> GlobalCounter {
>     counter.0 += inc_amount;
> 		counter.0
>   };
>   let value = context.query_world(query);
> # }
> ```
>
> Note the differences from a normal Bevy system:
>
> 1. The `query_world` method only accepts a single parameter, so your "system" should only accept that one. If you need more parameters, consider [creating a custom `SystemParam`](https://github.com/bevyengine/bevy/blob/9a7852db0f22eb41f259a1afbb4926eb73863a10/examples/ecs/system_param.rs#L21-L25).
> 2. The "system" may return a value

If you this code, though, you may notice that our `GlobalCounter` resource increments by 1, but then stops. Why is this? 

The reason this seems to only run once, is that we actually only run this code when the widget is rendered. If we called `set_inc_amount(2)`, for example, that would cause a state update, which would result in a re-render, which would result in the query being called again.

Because this query is called on every re-render (including when a parent updates), you'll need to be careful. It's probably best to add some logic that prevents the resource from being mutated on *every* re-render.

### Binding Resources

You may want your widget to bind to the resource directly, react to changes, and update accordingly. This can be done by wrapping a `Binding<T>` around our resource.

```rust
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
// A standard Bevy system that updates our counter
fn update_counter(mut counter: ResMut<Binding<GlobalCounter>>) {
  let current_value = counter.get().0;
  counter.set(GlobalCount(current_value + 1));
}
```

> The `counter` parameter does not actually need to be `mut` nor does it need to be typed as `ResMut`. This is because setting a binding uses interior mutability, thus not needing mutable access. However, Bevy's scheduler won't know this if you simply use `Res`. This can cause data races and result in undefined behavior. Therefore, it's best to still mark them as mutable.

#### Limitations

Todo: Talk about the limitations of using a `Binding`, including:

* clone
* partialeq
* construction
