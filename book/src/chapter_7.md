# Chapter 7 - Widget Update Systems(Diffing)
By default Kayak offers two widget update systems. These systems look at components on widgets and automatically diff(compare) them to the same components from the last render. If the widgets have changed it causes the internal systems to call that widget's render function.

The default widget update systems are:
- `widget_update<Props, State>` - This system diffs the props and state components. It also diffs some commonly provided components like: `KStyle` and `KChildren` automatically. Note: Only one type of state is allowed. In the future it might be possible to pass in a tuple of state types instead. If you require multiple state "types" please create a custom widget update function.
- `widget_update_with_context<Props, State, Context>` - Like `widget_update` but also provides functionality for auto diffing context as well. Again like with state this will only auto check one singular context type. 

## How does it work?
Behind the scenes Kayak UI keeps track of the types that are associated with props and state for a given widget. After each successful render of a widget kayak will clone the entire widget onto a new entity. This is considered the "last" render state of the entity and is expect to not change. These special entities can be avoided by using the `PreviousWidget` tag component and bevy query filters. They are also not added to the tree and are only loosely attached to the widget entity that lives in the tree.

## Custom widget update systems
Since the widget update is a system users can define very fine grained and custom diffing by writing their own system.

Kayak UI provides a bevy `SystemParam` called `WidgetParam` which aids in achieving this. The `SystemParam` takes two generics like the default widget update system for props and state. It has a special function which compares the components from entity A with entity B called `has_changed`.

## **WARNING!** - There is currently little documentation for how this works, and this is considered a rather delicate and highly advanced operation. 
Feel free to reach out if you feel like you would like to help improve this API!