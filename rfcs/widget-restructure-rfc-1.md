## Background

So I'm working on improving the focus management and I came across an idea. However, not one that I felt comfortable just dropping a PR for without opening it up to discussion first (since it could just be a horrendous idea). So here's the problem I found and a possible solution I came up with.

## Problem

A lot of internal stuff is exposed to the end user that either doesn't need to be or shouldn't be. At best it can be confusing or misleading to users, and at worst it can cause some users to use code in ways they shouldn't (i.e., unintentional hacks that are unstable and could break between versions).

For example, it seems wrong that a user can do this: `self.set_id(Index::default())`. And while this might not be that big of an issue ( "users should just *not* do that" ), what if we want to add other functionality onto widgets in the future? Maybe we add the ability to give widgets their own classes for theming. This might be confusing for the user who thinks that `self.add_class("foo")` should work on its own (but might not since the context also needs to  be made aware of the change).

Additionally, `KayakContext`, has a lot of functionality that doesn't need to be exposed within the context of a widget, but needs to elsewhere (such as the `KayakContext::render` and `KayakContext::process_events` methods). Methods like these need to be public, but they're dangerous to use within a widget's render method.

So in summary the two main issues are:

1. Internal widget data is exposed to the user in the render function in a confusing manner
2. Error-prone `KayakContext` methods are also exposed to the user in the render function when they shouldn't be

## Solution *(Version 3)*

> For Version 1, check out this commit: [841ff97fa68](https://github.com/StarArawn/kayak_ui/pull/56/commits/841ff97fa68b0e3919a42f871034383de10b2f68)
>
> For Version 2, check out these commits: [841ff97fa68..9a246ee1a](https://github.com/StarArawn/kayak_ui/pull/56/files/841ff97fa68b0e3919a42f871034383de10b2f68..9a246ee1a8fbc1e543f6ae58d5a18416f9e20dbf)

### Recap

#### Version 1

This version proposed changes that would add a layer of abstraction between a functional component, its generated widget struct, and the `KayakContext`. This was good for people writing functional components, but offered nothing for those defining their widgets manually.

It also added the concept of placing all user-defined props in a generated `<Widget Name>Props` struct to help isolate user-defined data with Kayak-defined data.

#### Version 2

Version 2 sought to take the concepts of Version 1 and apply some of them across manually defined widgets as well. To do this, the `KayakContextRef` was created, which acts as a temporary interface to `KayakContext` and is what would be passed to `Widget` implementors, instead of the raw context. Functional widgets would continue to use the stricter `<Widget Name>Context` to interface.

It also suggested bringing a props struct to manual implementors as well. To do this nicely it proposed a `WidgetConstructor` trait, which defined an associated `Props` type. After implementing, a widget could be created by calling `MyWidget::construct(id, children, props)`. And `props` would be generated like this `MyWidget::create_props(base_props)`.

This worked but created a disparity between widgets: a widget could choose to not impl `WidgetConstructor`, preventing it from being composed in tags (i.e., `<MyWidget />`). It also forced the `Props` type to return a default struct (not necessarily by implementing `Default`), which isn't well-suited for required props.

### Widget

#### KayakContextRef

The first issue to address is providing safe access to `KayakContext`. By "safe," I'm referring to the fact that currently access to `KayakContext` is unfiltered. This allows the user to run code they shouldn't or come up with unmaintainable hacks. To reduce this, we can add a struct that is temporarily created by `KayakContext` to act as a safer interface to itself. 

This is where `KayakContextRef` comes in:

```rust
struct KayakContextRef<'a> {
  context: &'a mut KayakContext,
  current_id: Index,
  // other fields...
}

impl<'a> KayakContextRef<'a> {
  pub(crate) fn new(context: &mut KayakContext, current_id: Index) {
    // ...
  }
  
  /// Expose the bind method to the widget
  pub fn bind(...) {
    /// Forward the call
    self.context.bind(...)
    // We can also perform other side-effects when this is called,
    // like updating a counter for how many bindings are created
  }

  /// This also allows us to update some KayakContext methods to be a bit more "pure".
  /// For example, we can update `KayakContext::create_state` to take an Index, instead
  /// of just assuming it's the current widget (for greater flexibility)
  pub fn create_state(...) {
    // Here, we pass current_id
    self.create_state(self.current_id, ...)
  }
}
```

#### Method Changes

This changes what `Widget::render` looks like:

```rust
trait Widget {
  // ...
  fn render(&mut self, context: &mut KayakContextRef);
}
```

Allowing it to be called like:

```rust
let widget = self.widget_manager.take(widget_id);

// Create temporary interface 
let mut context = KayakContextRef::new(self, widget_id);
widget.render(&mut context);

self.widget_manager.repossess(widget);
```

> A block *may* need to surround the call to `widget.render(&mut contex)` in order to drop `context` and appease the borrow checker.

And the same for the `Widget::on_event` method:

```rust
trait Widget {
  // ...
  fn on_event(&mut self, context: &mut KayakContextRef, event: &mut Event);
  // ...
}
```

##### Version 3: `mut` → `&mut`

One change from the previous versions is that we now pass `&mut KayakContextRef` instead of `mut KayakContextRef`. The reason for this is that it allows us to run additional code over the context after being used by the widget. For example, we may want to track changes or set flags using `KayakContextRef`. This change allows us to do that and process them after the fact.

#### WidgetProps

Another addition is the `Props` associated type:

```rust
trait Widget {
  type Props: WidgetProps;
  // ...
}
```

The `WidgetProps` trait will look something like this:

```rust
trait WidgetProps: Debug {
  fn get_on_event(&self) -> Option<OnEvent>;
  fn get_styles(&self) -> Option<Style>;
  fn get_focusable(&self) -> Option<bool>;
  // ...
}
```

##### Deriving

One additional feature for `WidgetProps` this RFC would like to propose is the ability to derive `WidgetProps`. This will make it much simpler for users to define their props and for defining Kayak-specific functionality.

The derive macro would have some associated macros, which will be used to mark certain fields for use in the implementation. These are:

* `#[props(OnEvent)]` - Used to mark a field as the `OnEvent` prop
* `#[props(Styles)]` - Used to mark a field as the `Styles` prop
* `#[props(Focusable)]` - Used to mark a field as the `Focusable` prop

There may be more added to this list in the future, but for now this is the main assortment.

There should only be a single usage of each of these markers within a struct. If there are more than one, only the one defined last will be used (last was chosen so we don't have to check if it already exists in the macro function).

If any of these attributes are missing, the relevant method will return none.

Additionally, they should be allowed to be specified as `Optional`. 

```rust
#[derive(WidgetProps)]
struct MyWidgetProps {
  #[props(OnEvent)]
  event_handler: OnEvent
  #[props(Focusable)]
  focusable: Optional<bool>,
  // Defined without the marker attribute:
  styles: Styles,
}

// Generated:
impl WidgetProps {
  fn get_on_event(&self) -> Option<OnEvent> {
  	Some(self.event_handler.clone())  
  }
  
  fn get_styles(&self) -> Option<Style> {
    None
  }
  
  fn get_focusable(&self) -> Option<bool> {
    self.focusable
  }
  
  // ...
}
```

#### The Constructor

Another addition to `Widget` would be the constructor method. This associated method will allow widgets to be generated with a set of props in a more controlled and freeform way. The method looks like this:

```rust
fn constructor(id: Index, children: Children, mut props: Self::Props) -> dyn Widget<Props=Self::Props> where Self: Sized;
```

> Note that this would put the `Sized` restriction on widgets.

Doing it this way allows the user to define the naming and placement of their props, or even replace them with their own.

---

### Defining Widgets

#### Custom Widgets

A custom widget is a widget defined manually by a user using standard Rust syntax (i.e. not by using the `#[widget]` attribute). By defining a widget manually, a user has finer control over behavior, data, and more.

To define a custom widget, we need to first define our struct:

```rust
#[derive(Debug)]
struct MyButton {
  my_id: Index,
  my_children: Children,
  my_props: MyButtonProps,
}
```

> We're naming these fields with the `my_` prefix to show that naming conventions are not mandatory for field names.

##### Props

While `Index` and `Children` are structs defined by Kayak, `MyWidgetProps` is not. This struct must be defined by the user and should contain all the props this widget expects.

One vital bit about this struct is that it is the only part of this RFC that requires a **specific naming convention**. All props for a widget must be defined as `<Widget Name>Props`. So `Foo` contains `FooProps` and `Props` (an awful name for a widget) contains `PropsProps`.

This is one of the major limitations to this system. Once the [`more_qualified_paths`](https://github.com/rust-lang/rust/issues/86935) feature is stabilized, we can remove this limitation by instead doing `<MyButton as Widget>::Props { ... }`. Unfortunately, this feature is only available in nightly at the moment.

For now though we can create our props struct and derive `WidgetProps` as explained above:

```rust
#[derive(Debug, WidgetProps)]
struct MyButtonProps {
  // Kayak Props
  #[props(OnEvent)]
  event_handler: Option<OnEvent>,
  #[props(Styles)]
  styles: Option<Style>,
  #[props(Focusable)]
  focusable: bool,
  
  // Widget Props
  text: String,
  disabled: bool,
}
```

##### Implementing Widget

Now with everything defined, we're ready to implement `Widget`.

Firstly, the constructor:

```rust
impl Widget for MyButton {
  type Props = MyButtonProps;
  
  fn constructor(id: Index, children: Children, mut props: Self::Props) -> dyn Widget<Props=Self::Props> where Self: Sized {
    if props.disabled {
      // If disabled, also disable focusability
      props.focusable = false;
    }

    Self {
      my_id: id,
      my_children: children,
      my_props: props
    }
  }
  
}
```

Then we can implement all the getters and setters:

```rust
fn get_props(&self) -> &Self::Props {
  &self.my_props
}

fn get_props_mut(&self) -> &mut Self::Props {
  &mut self.my_props
}

fn get_id(&self) -> Index {
  self.my_id
}

fn set_id(&mut self, id: Index) {
	self.my_id = id;  
}

fn get_name(&self) -> String {
  String::from("MyButton")
}
```

Lastly, the render method.

To help speed this last part up, another change will be proposed. This change actually targets another addition from this RFC: `KayakContextRef`.

##### Extending KayakContextRef

Normally to render out a widget, we need to do something akin to this:

```rust
// fragment.rs

fn render(&mut self, context: &mut KayakContext) {
  let parent_id = self.get_id();
  let tree = crate::WidgetTree::new();
  tree.add(parent_id, None);

  if let Some(children) = self.children.take() {
      children(tree.clone(), Some(parent_id), context);
  } else {
      return;
  }

  // Consume the widget tree taking the inner value
  let tree = tree.take();

  // Evaluate changes to the tree.
  let changes = context.widget_manager.tree.diff_children(&tree, parent_id);

  context.widget_manager.tree.merge(&tree, parent_id, changes);
}
```

But without access to `KayakContext` we can't quite do this anymore (which is good because it's a lot of boilerplate to write).

To address this and clean up the API a bit more, some more code will be placed in `KayakContextRef`.

```rust
struct KayakContextRef<'a> {
  // ...
  widget_tree: WidgetTree
}

impl<'a> KayakContextRef<'a> {
  pub(crate) fn new(context: &mut KayakContext, current_id: Index) {
    let widget_tree = WidgetTree::new();
    widget_tree.add(current_id, None);
   	Self {
      context,
      current_id,
      widget_tree,
    }
  }
  
  // Pretend `Children` isn't wrapped by `Option<...>`
  pub fn add_children(&mut self, children: Children) {
    let id =  self.current_id;
    children(self.widget_tree.clone(), Some(id), context);
  }

  pub fn add_widget<W: Widget>(&mut self, widget: W, widget_index: usize) {
    let id =  self.current_id;
    let (should_rerender, child_id) = self.context
      .widget_manager
      .create_widget(widget_index, widget.clone(), Some(id));
    self.widget_tree.add(child_id, Some(id));
    if should_rerender {
        let mut child_widget = self.context.widget_manager.take(child_id);
        let mut context = KayakContextRef { context: self.context, current_id: id };
        child_widget.render(context);
        self.context.widget_manager.repossess(child_widget);
    }
  }

  pub fn commit(mut self) {
    let tree = self.widget_tree.take();
    let id =  self.current_id;
    let changes = self.context.widget_manager.tree.diff_children(&tree, id);
    self.context.widget_manager.tree.merge(&tree, id, changes);
  }
}
```

> This was added in Version 2 of this RFC, but Version 3 internalizes a bit more of the logic, specifically by storing the `WidgetTree` in context.

And results in usage like so:

```rust
// fragment.rs

fn render(&mut self, mut context: KayakContextRef) {
  if let Some(children) = self.children.take() {
    context.add_children(children);
  }
  
  context.commit();
}

// vec.rs

fn render(&mut self, mut context: KayakContextRef) {
  for (index, item) in self.data.iter().enumerate() {
      context.add_widget(item, index);
  }
  
  context.commit();
}
```

<br />

<br />

#### Functional Widgets

The other form of widget we need to consider are functional widgets. These are the widgets to use when customization isn't a huge concern and you just need to define a working widget, and this is probably what most people will use when defining their own widgets.

In order to allow custom widgets to properly integrate with functional widgets and vice-versa, there are a few changes that need to be made.

Consider the following widget:

```rust
#[widget]
fn MyWidget(foo: Option<i32>, bar: String) {
  // ...
}
```

Currently this generates the following code:

```rust
#[derive(Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
struct MyWidget {
	pub id: kayak_core::Index,
  pub foo: Option<i32>,
  pub bar: String,
  #[derivative(Default(value = "None"))]
	pub styles: Option<Style>,
  #[derivative(Debug = "ignore", PartialEq = "ignore")] 
  pub children: Children,
	#[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")] 
  pub on_event: Option<kayak_core::OnEvent>,
}
```

However, we need this:

```rust
#[derive(Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
struct MyWidget {
  pub id: kayak_core::Index,
  #[derivative(Debug = "ignore", PartialEq = "ignore")] 
  pub children: Children,
  pub props: MyWidgetProps,
}
```

In order to properly get the props struct, we can instead define our widget like so:

```rust
#[widget]
fn MyWidget(props: MyWidgetProps) {
  let WidgetProps {foo, bar} = props;
  // ...
}

#[derive(WidgetProps)]
struct MyWidgetProps {
  foo: Option<i32>,
  bar: String
}
```

The props have been moved out and into their own struct, controlled entirely by the user. While this might be a bit more verbose for small widgets, it becomes much more manageable for widgets with a high number of props. It also allows methods to be defined for `MyWidgetProps`, which can be very useful for encapsulating custom logic.

> While it might not be necessary to enforce the `<Widget Name>Props` rule for a functional widget, since we can just get it from the type, it will probably be best to still follow that convention. Even going so far as to continue throwing errors like with custom widgets. This should help improve consistency across the two methods of defining a widget.

##### MyWidgetContext

All is normal so far. But something I suggested in Version 1 was to create a "Widget Context" that further separates widget logic from render logic (protecting the user from writing broken code or running into common pitfalls). This Widget Context, like `KayakContextRef`, would act as an interface for the render logic to use. Therefore, writing functional widgets is much safer, cleaner, and simpler, but comes at the cost of control. If more control and customization is needed, a manually defined custom widget is probably a better solution.

This context would look something like:

```rust
struct MyWidgetContext<'a> {
  context: &'a mut KayakContextRef,
  widget: &'a mut MyWidget,
}

impl<'a> MyWidgetContext<'a> {
  pub fn new(context: &mut KayakContextRef, widget: &mut MyWidget) -> Self {
    Self {
      context,
      widget,
    }
  }
  
  // Widget-specific logic and forwarding calls to KayakContextRef
}
```

The generated `MyWidget` would contain an associated function where all the render logic actually lives:

```rust
impl MyWidget {
  pub fn render_internal(context: &mut MyWidgetContext) {
    // ...
  }
}
```

And it would be called from the actual `Widget::render` method like so:

```rust
impl Widget for MyWidget {
  // ...
  fn render(&mut self, context: &mut KayakContextRef) {
    
    {
      let mut context = MyWidgetContext::new(context, self);
      Self::render_internal(context);
    }
    
    context.commit();
  }
  // ...
}
```

> Remember, this is all generated by the macro, a user would not be expected to write any of this.

<br />

---

## Tl;DR

1. Add `KayakContextRef` and update `Widget` methods - to interface with  `KayakContext` safely
2. Add `WidgetProps` trait and derive macro - to consistently define a widget's props
2. Move the props into its own struct for functional widgets
3. Generate Widget Context during `rsx` expansion - to interface with `KayakContext` and the widget safely

## Issues

### Protection

One issue with this implementation is that both manual and constructed widgets still have areas that lack protection. While we can no longer run arbitrary methods from `KayakContext`, we're still able to call things like `self.set_id(Index::default())`. This leaves us protected only on one side.

However, this might be okay if we see this as A Feature™. I definitely think this is important for manual widgets, who may need full access to data on their defining struct. Even constructed widgets may want more control over their data. So if we're okay with this, then it should be fine.

### Context Differences

On this topic, it may be worth noting that the different render methods between functional widgets and the others might be confusing. The former takes a `<WidgetName>Context` and the latter takes a `KayakContext`. This may be a minor issue, but I can see some users at least initially being confused by this difference if they switch from one to the other.

And we can't (or maybe shouldn't) just make both take `KayakContext` because the render method may need immutable access to the props, ID, focusability, etc.

One way to mitigate this is just to try and mimic as much of the `KayakContext` API as possible. This shouldn't be too difficult if we impl `Deref` and `DerefMut` over it. But the smaller the API difference, the less of an issue this becomes.

### Naming Requirements

As previously mentioned, we currently have a requirement on the naming of a struct for widget props. This must follow the `<Widget Name>Props` convention, otherwise the user will face compile errors. This can be solved once the  [`more_qualified_paths`](https://github.com/rust-lang/rust/issues/86935) feature is stabilized.

Other solutions are possible and were previously suggested, but they unfortunately had their own limitations and caveats. Namely, they put strict requirements on both the props and the `Widget` trait.

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

## Feedback

This is all just me thinking of ways to improve on these issues. Are they worth the trouble? And would we stand to gain from implementing a solution like this? I think so, but I could be totally and completely wrong. There could also be other, better solutions than the one I came up with. 

Anyways, just wanted to bring this up and see what people thought. So let me know!

## Addendum

### Tracking Changes

An additional feature this might allow is tracking changes that can't be tracked during the render phase:

```rust
#[widget]
fn MyWidget(disabled: bool) {
  context.set_focusable(disabled);
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
        on_event(&mut this, event);
      }
    }

    // Maybe something like this? (assuming it all compiles like it does in my head)
    let edits = this.edits;
    context.process_changes(edits);
  }
  // ...
}
```
