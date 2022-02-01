# Cursor Events

This section pertains to events related to the cursor.

>  **Note:** "cursor," "pointer," and "mouse" are sometimes used interchangeably in this context, even though they are technically unique from each other.

## Event Types

### Click

---

**Propagates** - ✅

**Inner Type** - `CursorEvent`

This event is triggered when a widget that received an `EventType::MouseDown` also receives an `EventType::MouseUp`. 

The topmost widget under the cursor is designated as the target of this event.

### Hover

---

**Propagates** - ✅

**Inner Type** - `CursorEvent`

This event is triggered when the cursor is over the bounds of a widget. It is only called when the cursor moves. If the cursor is over the widget but does not move, the event will not be called.

The topmost widget under the cursor is designated as the target of this event.

### Mouse Down

---

**Propagates** - ✅

**Inner Type** - `CursorEvent`

This event is triggered when the cursor is pressed down while within the bounds of a widget.

The topmost widget under the cursor is designated as the target of this event.

### Mouse Up

---

**Propagates** - ✅

**Inner Type** - `CursorEvent`

This event is triggered when the cursor is released (i.e. no longer pressed) while within the bounds of a widget.

The topmost widget under the cursor is designated as the target of this event.

### Mouse In

---

**Propagates** - ❌

**Inner Type** - `CursorEvent`

This event is triggered when the cursor enters the bounds of a widget.

All widgets under the cursor are designated as targets of this event. Since it does not propagate, each widget will only ever receive the event in which it is the target.

### Mouse Out

---

**Propagates** - ❌

**Inner Type** - `CursorEvent`

This event is triggered when the cursor exits the bounds of a widget.

All widgets under the cursor are designated as targets of this event. Since it does not propagate, each widget will only ever receive the event in which it is the target.

## Opting Out

By default, all widgets can be designated as the target of a cursor event. This might not always be ideal. For example, an overlay might capture all click events when you really want them to be sent to the widgets underneath. To control this, you can utilize `PointerEvents` style.

The following styles are available:

| Variant        | Description                                                  |
| -------------- | ------------------------------------------------------------ |
| `All`          | Allows all pointer events on this node and its children (this is the default) |
| `SelfOnly`     | Allows pointer events on this node but not on its children   |
| `ChildrenOnly` | Allows pointer events on this node's children but not on itself |
| `None`         | Disallows all pointer events on this node and its children   |

## Capturing the Cursor

Say you want to create a custom slider widget and set it up like this:

```rust,noplayground
#[widget]
fn Slider() {

  let (is_dragging, set_is_dragging, ..) = use_state!(false);
  // The value of the slider, from 0 to window's width
  let (value, set_value, ..) = use_state!(0);
  
  // Event handler for the slider's thumb
  let thumb_events = OnEvent::new(move |ctx, event| match event.event_type {
    EventType::MouseDown(..) => {
        set_is_dragging(true);
    }
    EventType::MouseUp(..) => {
        set_is_dragging(false);
    }
    EventType::Hover(data) => {
        if is_dragging {
            let next_value = data.position.0.min(100.0);
            set_value(next_value);
        }
    }
    _ => {}
  });

  // ...
}
```

This might seem like it will work the way you want, but if you ran this, you'd notice an issue with it immediately. The `Hover` event is only fired on widgets which contain the cursor within its bounds. Moving the cursor outside this range means we lose the ability to slide the slider even if we're still holding it down.

Ideally, we'd have a way to send that hover event to the slider widget while we hold down the cursor. This is where capturing comes in.

Capturing the cursor forces all cursor events to be sent to the captor widget, setting it as the target. The event will still propagate as normal, but the target will always be that widget. When we're done with it, we can subsequently release the widget.

To do this, we can use `KayakContextRef::capture_cursor(...)` and `KayakContextRef::release_cursor(...)`.

```rust,noplayground
# #[widget]
# fn Slider() {
# 
#   let (is_dragging, set_is_dragging, ..) = use_state!(false);
#   // The value of the slider, from 0 to window's width
#   let (value, set_value, ..) = use_state!(0);
#   
#   // Event handler for the slider's thumb
  let thumb_events = OnEvent::new(move |ctx, event| match event.event_type {
    EventType::MouseDown(..) => {
        ctx.capture_cursor(event.current_target);
        set_is_dragging(true);
    }
    EventType::MouseUp(..) => {
        ctx.release_cursor(event.current_target);
        set_is_dragging(false);
    }
#     EventType::Hover(data) => {
#         if is_dragging {
#             let next_value = data.position.0;
#             set_value(next_value);
#         }
#     }
#     _ => {}
  });
# 
#   // ...
# }
```



> Even if you don't know which widget has captured the cursor, you can still force its release by calling `force_release_cursor()`
