# Provider/Consumer

One special way of passing data between widgets that hasn't been covered yet is through *providers*. Providers allow you to share data with a widget's descendants, which they may subscribe to by becoming a *consumer* of that data.

This is useful for when an unknown descendant (if any) should be given access to this data. For example, a theme might be a good use-case. Not every widget needs the theme, so passing it around as a prop is a bit cumbersome. In this case, it's better to have it as provided data that may be consumed at any point down the widget tree.

However, be mindful when using the provider/consumer pattern. It might be tempting to use it, even when regular props would suffice. This can lead to code that's more difficult to maintain and debug, so use it sparingly!

## Providers

You can create a provider by calling `KayakContextRef::create_provider(...)` with the data to provide.

```rust,noplayground
#[widget]
fn ThemeProvider(children: Children) {
    context.create_provider(Theme::default());
    rsx! { <>{children}</> }
}
#
# #[derive(Debug, Default, Clone, PartialEq)]
# struct Theme {
#     primary: Color,
#     secondary: Color,
#     background: Color,
# }
```

What this does is allow all descendants of this widget to access the provided data. And since this data is only available to its descendants, sibling widgets will not be able to access it.

Any data can be provided granted that it satisfies `Clone`, `PartialEq`, `Send`, and `Sync`. It must also have a `'static` lifetime (you can't provide references).

### Nested Providers

Providers can be nested with no problem. Consumers of the data will always draw from the nearest ancestor provider. This means that if a nested provider provides the same data as a provider higher up the tree, consumers

```
[1] i32 Provider
    ├── i32 Consumer (consumes 1)
    └── [2] bool Provider
            ├── i32 Consumer (consumes 1)
            └── [3] i32 Provider
                    ├── i32 Consumer (consumes 3)
                    └── bool Consumer (consumes 2)
```

## Consumers

In order to use a provider's data, a descendant widget must register itself as a consumer of that data.

```rust,noplayground
#[widget]
fn MyWidget() {
    let theme = context.create_consumer::<Theme>().unwrap_or_default();
		// ...
}
```

If there is no valid provider in the ancestry of the widget, the returned value will be `None`. If there is, a binding of the requested data will be returned.

This binding works exactly like [states](./state.md) do in that setting a new value will trigger a re-render. However, in this case, the state is bound to the provider widget, meaning that updating the state will cause the provider to re-render, which in turn causes all descendants of that widget (including the one that updated the state) to be re-rendered as well.

## Comparison with Globals

> **Note:** This section uses the Bevy integration as an example for globals, but any integration that offers access to global data would work just as well.

You might be wondering: *What benefits does a provider/consumer pattern have over simple globals?*

Globals might seem like a better alternative—I mean, they allow everyone access! However, one things globals do not do well with is specificity.

This is best explained with an example. Say you have a GUI that displays a player's game results in such a way that passing that data as props isn't ideal (though, in most cases it probably is). So you write something like this:

```rust,noplayground
#[widget]
fn GameResults(player: Player) {
  rsx! {
    <Results player={player} />
  }
}

#[widget]
fn Results(player: Player) {
  // Create our globals
  let world = context.get_global_state::<World>().unwrap();
  world.insert_resource(PlayerName::from(player));
  world.insert_resource(PlayerScore::from(player));
  
  rsx! {
    <>
		  <PlayerName />
		  <PlayerScore />
    </>
  }
}

#[widget]
fn PlayerName() {
  let player_name = context.query_world(
    move |player_name: Res<Binding<PlayerName>| player_name.clone()
  );
  context.bind(&player_name);
  
  let player_name = player_name.get();
  
  // ...
}

#[widget]
fn PlayerScore() {
  let player_score = context.query_world(
    move |player_score: Res<Binding<PlayerScore>| player_score.clone()
  );
  context.bind(&player_score);
  
  let player_score = player_score.get();
  
  // ...
}
```

Apart from this being poorly designed code, you're feeling pretty good about it. But here's the issue: now you want to add multiplayer support.

Adding multiplayer seems like a huge hassle since you now need to deal with two or more players needing access to their respective player data. Fortunately, this is where providers shine.

With providers, you could add a level of specificity to the shared player data so that all descendant widgets use the correct data. To do this, we first need to update the widgets that manage the data:

```rust,noplayground
#[widget]
fn GameResults(player_1: Player, player_2: Player) {
  // For simplicity, let's just consider two players
  rsx! {
    <>
      <Results player={player_1} />
      <Results player={player_2} />
    </>
  }
}

#[widget]
fn Results(player: Player) {
  context.create_provider(PlayerName::from(player));
  context.create_provider(PlayerScore::from(player));
  
  rsx! {
    <>
		  <PlayerName />
		  <PlayerScore />
    </>
  }
}
```

Then, we'll also need to update the consumers:

```rust,noplayground
#[widget]
fn PlayerName() {
  let player_name = context.create_consumer::<PlayerName>().unwrap();
  let player_name = player_name.get();
  
  // ...
}

#[widget]
fn PlayerScore() {
  let player_score = context.create_consumer::<PlayerScore>().unwrap();
  let player_score = player_score.get();
  
  // ...
}
```

And just like that we made `PlayerName` and `PlayerScore` relative to their player— global-ish, yet specific. We can now handle as many players as we want by just passing their data to `Results` since it now creates a provider instead of a global.

As you can see, the provider/consumer pattern allows us to share global-like data but add additional specificity so our widgets can be more modular and easier to maintain.
