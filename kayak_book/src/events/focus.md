# Focus Events

This section pertains to events related to the focus system.

## Event Types

### Focus

---

**Propagates** - ❌

**Inner Type** - N/A

This event is triggered when a widget receives focus.

### Blur

---

**Propagates** - ❌

**Inner Type** - N/A

This event is triggered when a widget loses focus.

## Focus System

Only one widget may be focused at a time. When a widget is focused, the previous focus is blurred. Furthermore, if a focused widget is blurred with no other focus set, the root widget will take focus, thus ensuring that there will always be a focused widget.

### Focus Navigation

Widgets set as focusable are added to a separate focus tree (the root node is always the root of this tree). The tree is used for cycling to the next or previous focusable widget, where navigation travels between parent/child edges before moving to sibling widgets. Also, navigation loops: navigating past the last focusable widget returns focus to the first (which again is the root widget).

The default action of a <kbd>Tab</kbd> press navigates to the next focusable widget, while the default action of <kbd>Shift</kbd>+<kbd>Tab</kbd> navigates to the previous.

## Focusability

Whether a widget can be focused or not is based on its *focusability*. Focusability has three values:

| Value         | Description                               |
| ------------- | ----------------------------------------- |
| `Some(true)`  | The widget is focusable                   |
| `Some(false)` | The widget is not focusable               |
| `None`        | The widget can be either focusable or not |

The focusability of a widget can be set one of three ways:

1. On the widget definition

   ```rust,noplayground
   #[widget(focusable)]
   fn MyWidget() {
     // ...
   }
   ```

2. Using `KayakContext`

   ```rust,noplayground
   # #[widget]
   # fn MyWidget() {
     let event_handler = OnEvent::new(move |ctx, event| match event.event_type {
       EventType::Click(..) => ctx.set_focusable(Some(true), event.current_target),
       _ => {}
     });
   # }
   ```

3. As a [common prop](../widgets/common_props.md#focusable) on the widget

   ```rust,noplayground
   # #[widget]
   # fn MyWidget() {
     rsx! {
       <SomeWidget focusable={Some(true)} />
     }
   # }
   ```

Methods 1 & 2 take precedence over Method 3. This means a parent can only set the focusability of a child if the child allows it to do so (by having a focusability of `None`).
