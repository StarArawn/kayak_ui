# Macros
Kayak UI has 4 different proc macros:

- widget - A macro that turns a functional widget into a widget struct.
- rsx - A macro that turns RSX syntax into structure constructors and calls the context to create the widgets.
- constructor - A macro that turns RSX syntax into structure constructors only.
- render! - A top level macro that works the same as RSX but provides some additional context for building the root widget.