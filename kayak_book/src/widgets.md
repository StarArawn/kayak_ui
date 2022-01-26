# Widgets

A widget can be defined as any object that lives in the UI tree. Typically a widget will have some sort of visual appearance, however that doesn't necessarily have to be the case you might have widgets that manage state or just wrap other widgets. In Kayak UI we have a set of default widgets you can use that are completely optional. A full list of widgets:

- [App](../../src/widgets/app.rs) - A top level widget that sizes your UI to the bevy screen width/height.
- [Background](../../src/widgets/background.rs) - A widget which renders a colored quad. 
- [Button](../../src/widgets/button.rs) - A simple button widget
- [Clip](../../src/widgets/clip.rs) - This widget provides a rectangular clip area. Widgets rendered as children of the Clip will get chopped off if they exceed the bounds of the clip.
- [Element](../../src/widgets/element.rs) - A widget who provides additional layout but renders nothing.
- [Fold](../../src/widgets/fold.rs) - A collapsing widget
- [If](../../src/widgets/if_element.rs) - A conditional widget that will render it's children depending on X condition.
- [Image](../../src/widgets/image.rs) - Renders the provided image.
- [Inspector](../../src/widgets/inspector.rs) - A special widget which allows you to "inspect" the tree by clicking on widgets.
- [NinePatch](../../src/widgets/nine_patch.rs) - A nine patch renderer widget. Nine patches are nine textures(atlas single image) that represent 9 different sections of a rectangle: corners, in-between corners, and the middle.
- [TextBox](../../src/widgets/text_box.rs) - A widget which allows typing.
- [Text](../../src/widgets/text.rs) - A widget that displays text.
- [Tooltip](../../src/widgets/tooltip.rs) - A widget that renders a popup that displays its children.
- [Window](../../src/widgets/window.rs) - A draggable box with a title and children.

# Creating your own widgets
With Kayak UI you can create widgets using two different methods. You can create them using the widget proc macro or by manually implementing `Widget` on a struct.

## Functional widget using a proc macro
This is by far the simplest method, but also can be difficult at times due to the borrow checker in rust and prop passing. In the future this should get better! Here is a simple example of creating your own widget:

```rust
#[widget]
pub fn MyWidget(children: Children) {
    rsx! {
        <> // Empty arrow brackets like this are considered Fragments.
            {children}
        </>
    }
}
```

Since this is a proc macro it's important to understand what happens here. When the proc macro expands it looks something like this:

```rust
#[derivative(Default, Debug, PartialEq, Clone)]
pub struct Element {
    pub id: kayak_core::Index,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    pub children: Children,
    #[derivative(Default(value = "None"))]
    pub styles: Option<kayak_core::styles::Style>,
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub on_event: Option<kayak_core::OnEvent>,
    #[derivative(Default(value = "None"))]
    pub focusable: Option<bool>,
}

 impl kayak_core::Widget for Element {
    fn get_id(&self) -> kayak_core::Index {
        self.id
    }
    fn focusable(&self) -> Option<bool> {
        self.focusable
    }
    fn set_id(&mut self, id: kayak_core::Index) {
        self.id = id;
    }
    fn get_styles(&self) -> Option<kayak_core::styles::Style> {
        self.styles.clone()
    }
    fn get_name(&self) -> String {
        String::from("Element")
    }
    fn on_event(
        &mut self,
        context: &mut kayak_core::context::KayakContext,
        event: &mut kayak_core::Event,
    ) {
        if let Some(on_event) = self.on_event.as_ref() {
            if let Ok(mut on_event) = on_event.0.write() {
                on_event(context, event);
            }
        }
    }
    fn render(&mut self, context: &mut kayak_core::context::KayakContext) {
        // It's important to set the current ID this is how we track state management.
        let parent_id = self.get_id();
        context.set_current_id(parent_id);
        let parent_id = Some(parent_id);

        // We pull out the props(struct fields) from the widget here.
        let Element {
            children, ..
        } = self;

        let children = children.clone();

        // We create a new sub tree.
        let tree = kayak_core::WidgetTree::new();
        {
            // Creates a new fragment widget which renders the children. 
            let built_widget = ::kayak_core::Fragment {
                children: children.clone(),
                styles: None,
                on_event: None,
                ..Default::default()
            };

            // Creates the fragment widget.
            let (should_rerender, child_id) =
                context
                    .widget_manager
                    .create_widget(0usize, built_widget, parent_id);
            // Add the fragment widget to the sub tree.
            tree.add(child_id, parent_id);
            
            // If should_rerender is true we need to render the widget.
            if should_rerender {
                // We pull the widget out of storage
                let mut child_widget = context.widget_manager.take(child_id);
                // Render the widget
                child_widget.render(context);
                // And place it back into storage.
                context.widget_manager.repossess(child_widget);
            }
        }
        // Take ownership of the sub tree.
        let tree = tree.take();

        // Diff the sub tree with the full UI tree.
        let changes = context
            .widget_manager
            .tree
            .diff_children(&tree, self.get_id());

        // Apply the changes to the full UI tree.
        context
            .widget_manager
            .tree
            .merge(&tree, self.get_id(), changes);
    }
}
```

You can see that the functional widget turns into a standard rust struct. Note: A lot of the complexity here is to be replaced by a simpler system please see: [RFC1](../../rfcs/widget-restructure-rfc-1.md). This will allow people to make struct based widgets in a much more user friendly way.