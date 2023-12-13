# Chapter 5 - Context
Context in Kayak UI allows users to pass data through the widget tree without having to pass data between each widget at every level in the tree.

Typically in Kayak the data is passed top-down or parent to child via "props", but this can be cumbersome when dealing with complex widget trees. Context provides a way to share values from a parent widget down to children without the need to explicitly pass that data through every level of the tree.

## When to use context?
Context can be great for sharing "global" data. This might be something as simple as a user or potentially even more complex data. In the [tabs example](../../examples/tabs/tabs.rs) we can see how context is used to pass shared tab state to the buttons and the button content.


## How to use context?
Context starts by creating a context entity that is associated with a piece of data and it's parent widget. This looks like:
```rust
// Check if the context entity already exists
if widget_context
    .get_context_entity::<MyContext>(entity)
    .is_none()
{
    // Spawn the context entity with initial state
    let context_entity = commands
        .spawn(MyContext {
            foo: 0,
        })
        .id();

    // Let the widget context know about our custom context data.
    widget_context.set_context_entity::<MyContext>(Some(entity), context_entity);
}
```

Once the context has been created users can query the context in widget render systems like so:
```rust
fn my_child_widget_render(
    ...
    context_query: Query<&MyContext>,
) {
    // get_context_entity will jump up the tree until it finds a matching data type and entity.
    if let Some(context_entity) = widget_context.get_context_entity::<MyContext>(entity) {
        let context_data = context_query.get(context_entity);
        dbg!(context_data.foo);
    }
}
```

## Handling diff and widget updating
When a widget is associated with context it's important that the widget update system is aware of this. By default the `widget_update` diff system is used. This accepts types for props and state but not for context. A separate system called `widget_update_with_context` can be used which takes an optional third type for context.

### Important!
`widget_update_with_context` is only designed to work with one type of context. If you need multiple context diffing for a single widget it's advised that you use a custom widget update system.
