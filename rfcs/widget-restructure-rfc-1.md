## Background

So I'm working on improving the focus management and I came across an idea. However, not one that I felt comfortable just dropping a PR for without opening it up to discussion first (since it could just be a horrendous idea). So here's the problem I found and a possible solution I came up with.

## Problem

A lot of internal stuff is exposed to the end user that either doesn't need to be or shouldn't be. At best it can be confusing or misleading to users, and at worst it can cause some users to use code in ways they shouldn't (i.e., unintentional hacks that are unstable and could break between versions).

For example, it seems wrong that a user can do this: `self.set_id(Index::default())`. And while this might not be that big of an issue ( "users should just *not* do that" ), what if we want to add other functionality onto widgets in the future? Maybe we add the ability to give widgets their own classes for theming. This might be confusing for the user who thinks that `self.add_class("foo")` should work on its own (but might not since the context also needs to  be made aware of the change).

Additionally, `KayakContext`, has a lot of functionality that doesn't need to be exposed within the context of a widget, but needs to elsewhere (such as the `KayakContext::render` and `KayakContext::process_events` methods). Methods like these need to be public, but they're dangerous to use within a widget's render method.

So in summary the two main issues are:

1. Internal widget data is exposed to the user in the render function in a confusing manner
2. Error-prone `KayakContext` methods are also exposed to the user in the render function when they shouldn't be

## Solution

I think the widget system could be restructured in a way that creates a layer of abstraction between the widgets and their context.

Let's assume we have this widget:

```rust
#[widget]
fn MyWidget(name: String, fill: Option<Color>) {
  // ...
}
```

### Props Extraction

To get around the first issue of the widget having too much mutable access to its required widget data, we could separate the render function from the widget itself by placing the actual logic inside an associated method. 

This would remove the access to a mutable `self`. But we still need access to the actual props. To solve this issue we can generate a new struct, `MyWidgetProps`, to contain the prop data. We can then pass this into the render function.

And now we have something that looks a bit more like this:

```rust
struct MyWidget {
  pub id: Index,
  pub children: Children,
  pub props: MyWidgetProps,
  // ...
}

struct MyWidgetProps {
  pub name: String,
  pub fill: Option<Color>,
}

impl Widget for MyWidget {
  fn render(&mut self, context: &mut KayakContext) {
    Self::render(self.props, context);
  }
}

impl MyWidget {
  fn render(props: MyWidgetProps, context: &mut KayakContext) {
    // Generated logic
  }
}
```

### Widget Context

Now comes the second issue of direct access to `KayakContext`. I think a good way of solving this would be to generate an interface struct. This struct would act as a middleman between the render function and `KayakContext`, forwarding only select methods to the context.

There are two ways of going about this.

1. Create a local struct:

```rust
struct MyWidgetContext<'a> {
  context: &'a mut KayakContext,
  widget: &'a mut MyWidget
}
```

2. Create a one-size-fits all struct

```rust
struct MyWidgetContext<'a> {
  context: &'a mut KayakContext,
  widget: &'a mut dyn Widget
}
```

Though option 1 results in more code being generated, I actually like it better since it allows for greater customization in cases where the widget is written out manually instead of via cookie-cutter macro. 

Using this interface struct, we can then replace the direct `KayakContext` access:

```rust
impl MyWidget { 
  fn render(mut this: MyWidgetContext) {
    // Generated logic
  }
}
```

> We can of course call the parameter `context` or something. I'm just being cheeky by calling it `this` here ;)

And of course we'll need some code for it to be useable:

```rust
/// We could also create a trait for common methods, but this is probably fine
/// since we'll likely only ever use it internally within a particular widget
/// (in other words, not as a passable trait object)
impl<'a> MyWidgetContext<'a> {
  pub fn props(&self) -> &MyWidgetProps {
    &self.widget.props
  }
  
  pub fn set_styles(&mut self, styles: Option<Style>) {
    self.widget.styles = styles;
  }
  
  // ...
  
}

impl MyWidget {
  fn render(mut this: MyWidgetContext) {
    let MyWidgetProps {name, fill} = this.props();
    
    // Generated logic
  }
}
```

And again, the purpose here is to hide internals so the user isn't confused and doesn't code in a hacky way:

```rust
impl MyWidget {
  fn render(mut this: MyWidgetContext) {
    // It's now impossible to change a widget's ID internally
    self.set_id(Index::default());
    
    // You can no longer render-ception
    context.render();
    
    // This might seem like a hack to dispatch events but it's an error
    context.process_events(vec![InputEvent::MouseLeftPress]);
    
    // Now we can make this properly notify the context
    this.add_class("foo");
  }
}
```

### TL;DR

1. Generate `MyWidgetProps`
2. Move rendering to associated function
3. Create `MyWidgetContext` to interface between render function and `MyWidget` and `KayakContext`
4. ???
5. Profit

## Alternatives

### Faith

The biggest alternative, again, is just to trust that the developer isn't going to cause any issues and that we make it clear what code *could* cause issues. But I think we can still do something to help. And coming up with a solution also improves the overall experience and API.

### Data-less Widgets

One big alternative to a portion of this is to just not store data on the widget itself. All relevant data for a widget will be stored in the `KayakContext`. This forces all code that needs a bit of data about a widget to also need access to the current context.

This could make code outside the functional component much more cumbersome to use.

Additionally, we run into the issue of storing the widget's ID, since that data does need to exist on the widget regardless.

### Context Setters

One of the issues I briefly mentioned was about the idea that updating a widget's own data does not directly alert the context. This could be solved by just having users write the notification code:

```rust
#[widget]
fn Foo(context: &mut KayakContext) {
  self.add_class("bar");
  context.notify(KayakNotify::ClassAdded); // or however it's implemented
}
```

From an API perspective, this isn't great. Using the Widget Context idea, we can move that required code into the interface method:

```rust
impl<'a> FooContext<'a> {
  pub fn add_class(&mut self, class: &str) {
    self.widget.add_class(class);
    context.notify(KayakNotify::ClassAdded);
  }
}

#[widget]
fn Foo() {
  this.add_class("bar");
}
```

Also note that we can't do this:

```rust
#[widget]
fn Foo(context: &mut KayakContext) {
  context.add_class("bar");
}
```

Since this requires that `Foo` have a public method `add_class` for the context to set. And the alternatives for those issues go back to the ones listed above.

### Move Render Function into Widget Context

One potential alternative to what's mentioned above would be to move the associated render function from `MyWidget` into the `MyWidgetContext`. This would allow `self` to be used as normal and even give direct access to `context` and `widget` if needed, while still granting the separation that allows us to add custom logic.

## Sample Generated Output

<details>
<summary><em>Show/Hide Code</em></summary>

```rust
use crate::OnEvent;
use crate::render_command::RenderCommand;
use crate::styles::{Style, StyleProp};

mod my_widget_module {
    use derivative::Derivative;
    use crate::{Color, Index, KayakContext, OnEvent, Widget};
    use crate::styles::Style;

    #[derive(Derivative)]
    #[derivative(Default, Debug, PartialEq)]
    pub struct MyWidget {
        pub id: Index,
        pub props: MyWidgetProps,
        pub styles: Option<Style>,
        #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
        pub on_event: Option<crate::OnEvent>,
        #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
        pub children: crate::Children,
    }

    #[derive(Derivative)]
    #[derivative(Default, Debug, PartialEq)]
    pub struct MyWidgetProps {
        pub name: String,
        pub fill: Option<Color>,
    }

    pub struct MyWidgetContext<'a> {
        context: &'a mut KayakContext,
        widget: &'a mut MyWidget,
    }

    impl<'a> MyWidgetContext<'a> {
        pub fn set_styles(&mut self, styles: Option<Style>) {
            self.widget.styles = styles;
        }

        pub fn on_event(&mut self, handler: Option<OnEvent>) {
            self.widget.on_event = handler;
        }

        pub fn props(&self) -> &MyWidgetProps {
            &self.widget.props
        }
    }

    impl MyWidget {
        pub fn generate_context<'a>(&'a mut self, context: &'a mut KayakContext) -> MyWidgetContext<'a> {
            MyWidgetContext {
                widget: self,
                context,
            }
        }
    }

    impl Widget for MyWidget {
        fn get_id(&self) -> Index {
            self.id
        }

        fn focusable(&self) -> bool {
            true
        }

        fn set_id(&mut self, id: Index) {
            self.id = id;
        }

        fn get_styles(&self) -> Option<Style> {
            self.styles.clone()
        }

        fn get_name(&self) -> String {
            String::from("MyWidget")
        }

        fn on_event(&mut self, context: &mut KayakContext, event: &mut crate::Event) {
            let this = self.generate_context(context);
            if let Some(on_event) = this.widget.on_event.as_mut() {
                if let Ok(mut on_event) = on_event.0.write() {
                    on_event(this.context, event);
                }
            }
        }

        fn render(&mut self, context: &mut KayakContext) {
            Self::render(self.generate_context(context));
        }
    }
}

use my_widget_module::{MyWidget, MyWidgetContext, MyWidgetProps};
// Outside the module so it doesn't have access to the internals
impl MyWidget {
    pub fn render(mut this: MyWidgetContext) {
        let MyWidgetProps { fill, name } = this.props();

        this.set_styles(Some(Style {
            render_command: StyleProp::Value(RenderCommand::Empty),
            ..Default::default()
        }));

        this.on_event(Some(OnEvent::new(move |_, evt| {
            println!("Event fired!");
        })));

        // ...
    }
}
```

</details>

## Feedback

This is all just me thinking of ways to improve on these issues. Are they worth the trouble? And would we stand to gain from implementing a solution like this? I think so, but I could be totally and completely wrong. There could also be other, better solutions than the one I came up with. 

Anyways, just wanted to bring this up and see what people thought. So let me know!

## Addendum

### Tracking Changes

An additional feature this might allow is tracking changes that can't be tracked during the render phase:

```rust
#[widget]
fn MyWidget(disabled: bool) {
  this.set_focusable(disabled);
  // ...
}
```
And in `MyWidgetContext`:

```rust
pub fn set_focusable(&mut self, focusable: bool) {
  self.widget.focusable = focusable;

  // If we need to track this change, we could do something like:
  self.edits.insert(WidgetEdit::Focus);
}
```

Then if we need to handle any changes we can just iterate through the set of changes and handle them individually.

```rust
impl Widget for MyWidget {
  // ...
  fn on_event(&mut self, context: &mut KayakContext, event: &mut crate::Event) {
    let mut this = self.generate_context(context);
    if let Some(mut on_event) = this.widget.on_event.clone() {
      if let Ok(mut on_event) = on_event.0.write() {
        on_event(this, event);
      }
    }

    // Maybe something like this? (assuming it all compiles like it does in my head)
    let edits = this.edits;
    context.process_changes(edits);
  }
  // ...
}
```