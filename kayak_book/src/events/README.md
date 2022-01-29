# Events

Events are used to relay user input to a widget. This includes things like clicks and keyboard presses. We can pass that input to a widget, giving it a chance to react to the event in its own way, such as by printing something to the console or updating its state.

## Event Types

Each event is labeled with a particular `EventType`, which both identifies the event and contains data related to it. You can match the `EventType` your widget cares about and add specific logic to handle it.

One important note about `EventType` is that, while they do implement `PartialEq` and `Hash`, they only consider the discriminant part of the variant. Any inner data is therefore disregarded.

## Lifecycle

To fully understand Kayak's event system, it's important to first understand the lifecycle of an event. This includes how events are created, how they're processed, and how they're dispatched.

### Creation

Events start out as a completely separate event type called `InputEvent`. As you can guess, these events directly map to the raw inputs given. These input events are then sent to `KayakContext`, where they will be processed and dispatched immediately.

### Processing

The first step to processing input events is to find a **target** widget. The target widget of an event will be the one to receive the event. How this is determined is based on the type of event. For cursor events, the target is typically the topmost[^1] widget that the cursor is currently over. Whereas, keyboard events consider the currently focused widget to be the target.

Once a proper target is found, the input events can then be converted to a standard `Event` struct and sent off for dispatching.

### Dispatch

After processing, each event is then dispatched to its target widget. Once dispatched, the event may attempt to do two things:

1. Propagate
2. Perform a default action

#### Propagation

Most events are able to propagate. This means that once the target widget is done handling the event, it gets passed to the widget's parent, then passed to that widget's parent, and so on. At each point the `current_target` field is updated to match the current widget (the `target` field remains fixed on the original target).

Propagation can be easily stopped, however, by calling the event's `stop_propagation()` method. This will prevent the event from propagating to any ancestors, which may be useful when an event is fully handled by a widget. For example, a nested button can prevent the containing button from also being activated on click.

#### Default Actions

Some events actually have a default action. These actions are not configured by the user— or by any widget for that matter. Instead, they're set by Kayak. One example of a default action is on a <kbd>Tab</kbd> press. This default action simply navigates focus to the next focusable widget.Default actions are often based on convention with other technologies. 

And, like propagation, they can be prevented. To do this, simply call the event's `prevent_default()` method. For example, to completely stop the default action on a <kbd>Tab</kbd> press, you can add a handler on the root node that calls this method when the <kbd>Tab</kbd> key is pressed.

## Handling Events

All widgets— at least all functional widgets— allow you to attach an event listener to handle these input events. On most widgets this can be set by passing an `on_event` prop.

A handler should be an `Option<OnEvent>`, where the `OnEvent` struct contains your handler closure. The closure takes two parameters: a reference to the `KayakContext` and the `Event`.

```rust,noplayground
# #[widget]
# fn MyWidget() {
  let event_handler = OnEvent::new(move |ctx, event| {
    println!("Received Event: {:#?}", event);
    match event.event_type {
      EventType::Click(data) => {
        event.stop_propagation();
        println!("Click @ {:?}", data.position);
      },
      EventType::KeyDown(data) => {
        if(matches!(data.key(), KeyCode::Tab)) {
          event.prevent_default();
        }
      },
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

<br />

---

[^1]: The "topmost" widget is said to be a widget that is either the deepest child or has the highest z-index.
