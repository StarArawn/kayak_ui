# Chapter 4 - State
State is data that is directly associated with a particular widget. State persists during re-renders, but will be destroyed when an widget is despawned.

State can be created in Kayak easily, here is an example:
```rust
#[derive(Component, Default, PartialEq, Clone)]
struct CurrentCountState {
    foo: u32,
}

// During a widget's render function
let state_entity = widget_context.use_state(
    // Bevy commands
    &mut commands,
    // The widget entity.
    entity,
    // The default starting values for the state.
    CurrentCountState::default()
);

// State can be queried like any entity.
// This can be done via bevy query iteration or via lookup using the state_entity
if let Ok(state) = state_query.get(state_entity) {
    dbg!(state.foo);
}

fn my_bevy_system(state_query: Query<&CurrentCountState>) {
    for state in state_query.iter() {
        dbg!(state.foo);
    }
}
```

When an entity is despawned the state associated with that entity is also despawned.

## Update/Diffing system
By default Kayak provides a system for diffing widgets. This system is called: `widget_update` and can be attached to any widget via the root kayak context.

`widget_update` takes two generic parameters called: `Props` and `State`. Both of these types expect to derive: `Component`, `PartialEq`, and `Clone`. Currently only one state type can be defined which can be an issue as widgets might have more than one piece of state. For now it's advised to group state by component type, however if a user desires they can implement a custom widget update system which manually checks more pieces of state. This is however considered to be more advanced usage and may not be as well documented.

