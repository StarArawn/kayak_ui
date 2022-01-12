## Background

So I'm working on improving the focus management and I came across an idea. However, not one that I felt comfortable just dropping a PR for without opening it up to discussion first (since it could just be a horrendous idea). So here's the problem I found and a possible solution I came up with.

## Problem

A lot of internal stuff is exposed to the end user that either doesn't need to be or shouldn't be. At best it can be confusing or misleading to users, and at worst it can cause some users to use code in ways they shouldn't (i.e., unintentional hacks that are unstable and could break between versions).

For example, it seems wrong that a user can do this: `self.set_id(Index::default())`. And while this might not be that big of an issue ( "users should just *not* do that" ), what if we want to add other functionality onto widgets in the future? Maybe we add the ability to give widgets their own classes for theming. This might be confusing for the user who thinks that `self.add_class("foo")` should work on its own (but might not since the context also needs to  be made aware of the change).

Additionally, `KayakContext`, has a lot of functionality that doesn't need to be exposed within the context of a widget, but needs to elsewhere (such as the `KayakContext::render` and `KayakContext::process_events` methods). Methods like these need to be public, but they're dangerous to use within a widget's render method.

So in summary the two main issues are:

1. Internal widget data is exposed to the user in the render function in a confusing manner
2. Error-prone `KayakContext` methods are also exposed to the user in the render function when they shouldn't be

## Solution *(Version 2)*

> For Version 1, check out this commit: [841ff97fa68](https://github.com/StarArawn/kayak_ui/pull/56/commits/841ff97fa68b0e3919a42f871034383de10b2f68)

### Issues and Differences with Version 1

One issue with the solution proposed in Version 1 of this RFC, is that it only focused on those choosing to use `rsx`. Custom widgets would still need to go through a lot of boilerplate code in order to get up and running. Additionally, these custom widgets needed to match the schema of their `rsx` counterparts, down to the name (more-or-less).

This updated version attempts to target these users as well, since customizing a widget to your liking is definitely a great thing to have. I'll go through each section in order of increasing automation (from manually writing code to using mostly macros). But first, changes to `Widget` and how it renders.

---

###  Widget

Previously we had the `WidgetContext` act as the barrier between the render method and the widget/context. This works well for `rsx`-generated widgets, but not so much for manually defined widgets. The reason is that by using `WidgetContext` we place a lot more expectations on how the rendering should be done. Again, one of the goals with this RFC is to cut back on expectations placed on the user.

And while `WidgetContext` isn't ideal for these scenarios, it still provides a feature we want: a safe interface to the context. So this is the first change of Version 2: `KayakContextRef`.

#### KayakContextRef

This struct will be the interface into `KayakContext`. In it, we can place code specifically for widgets to use during their render phase.

```rust
struct KayakContextRef<'a> {
  context: &'a mut KayakContext,
  current_id: Index,
}

impl<'a> KayakContextRef<'a> {
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
  fn render(&mut self, mut context: KayakContextRef);
}
```

Allowing it to be called like:

```rust
let widget = self.widget_manager.take(widget_id);
let mut context = KayakContextRef { context: self, current_id: widget_id };
widget.render(context);
self.widget_manager.repossess(widget);
```

And the same for the `Widget::on_event` method:

```rust
trait Widget {
  // ...
  fn on_event(&mut self, mut context: KayakContextRef, event: &mut Event);
  // ...
}
```

That's it for the changes to the `Widget` trait!

---

### Defining Widgets

#### Manual Widgets

> **Note:** In my opinion, this method is the least likely to be utilized as it isn't very versatile. But it's included to showcase the polar opposite side of using the `rsx` macros. This already exists with `VecTracker` (to some degree).

Manual widgets are widgets that are not meant to be used with the tagging (`</>`) syntax. This is because they want to opt out of certain restrictions that tagged widgets have or to just completely take control.

Some possible reasons include:

* Avoiding implementing/deriving `Default`
* Using a custom schema (not the one used by other widgets)
* Needing only the bare essentials (i.e. just `impl Widget`)

```rust
#[derive(Debug, PartialEq)]
struct MyManualWidget {
  my_id: Index,
  custom_value: CustomFoo,
}
```

Here, we create the widget with non-standard fields and without deriving `Default` (though `PartialEq` isn't a bound on the `Widget` trait, it seems some of the macros still rely on it to exist).

Its goal is to be invoked like so:

```rust
rsx! {
  <Element>
  	{
    	  MyManualWidget::new(CustomFoo(123))
  	}
  </Element>
}
```

To do this, it has to implement `Widget`:

```rust
impl Widget for MyManualWidget {
  fn get_id(&self) -> Index { self.my_id }
  // ...
  fn render(&mut self, mut context: KayakContextRef) {
    // ???
  }
}
```

Currently, the only way to build this widget is to do something like:

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

There are two options for handling this: move logic into `KayakContextRef` or create a `WidgetTreeBuilder`.

##### Extending KayakContextRef

This would add the following methods:

```rust
pub fn create_tree(&self) -> WidgetTree {
  let tree = WidgetTree::new();
  tree.add(self.current_id, None);
  tree
}

// Pretend `Children` isn't wrapped by `Option<...>`
pub fn add_children(&mut self, children: Children, tree: &WidgetTree) {
	let id =  self.current_id;
	children(tree.clone(), Some(id), context);
}

pub fn add_widget<W: Widget>(&mut self, widget: W, widget_index: usize, tree: &WidgetTree) {
  let id =  self.current_id;
  let (should_rerender, child_id) = self.context
    .widget_manager
    .create_widget(widget_index, widget.clone(), Some(id));
  tree.add(child_id, Some(id));
  if should_rerender {
      let mut child_widget = self.context.widget_manager.take(child_id);
      let mut context = KayakContextRef { context: self.context, current_id: id };
      child_widget.render(context);
      self.context.widget_manager.repossess(child_widget);
  }
}

pub fn commit_tree(&mut self, tree: WidgetTree) {
  let tree = tree.take();
  let id =  self.current_id;
  let changes = self.context.widget_manager.tree.diff_children(&tree, id);
  self.context.widget_manager.tree.merge(&tree, id, changes);
}
```

And results in usage like so:

```rust
// fragment.rs

fn render(&mut self, mut context: KayakContextRef) {
	let tree = context.create_tree();
  if let Some(children) = self.children.take() {
    context.add_children(children, &tree);
  } else {
      return;
  }
  context.commit_tree(tree);
}

// vec.rs

fn render(&mut self, mut context: KayakContextRef) {
	let tree = context.create_tree();
  
  for (index, item) in self.data.iter().enumerate() {
      context.add_widget(item, index, &tree);
  }
  
  context.commit_tree(tree);
}
```

##### WidgetTreeBuilder

Alternatively, we could create a `WidgetTreeBuilder` struct that is use specifically for doing everything as above. The benefit of using the builder is that we can move some of the render specific logic to its own separate entity (again separating the concerns). It could be generated from `KayakContextRef::create_tree` and have a similar API:

```rust
struct WidgetTreeBuilder<'a> {
  pub tree: WidgetTree,
  context: &'a mut KayakContext,
  current_id: Index,
}

impl<'a> WidgetTreeBuilder<'a> {
	pub fn add_children(&mut self, children: Children) -> &mut WidgetTreeBuilder {
    // ...
  }
  
  pub fn add_widget<W: Widget>(&mut self, widget: W, widget_index: usize) -> &mut WidgetTreeBuilder {
    // ...
  }
  
  pub fn commit(self) {
    // ...
  }
}
```

> Not super sold on the name of `WidgetTreeBuilder` since it's not really building anything?

<br />

<br />

#### Constructed Widgets

A constructed widget, for the purposes of this RFC, is one that is manually defined like above but uses some traits and derives to allow it to be used as a tagged (`</>`) component. Like the manually defined widgets, it's good for adding customization and extending the widget itself with other functionality.

To integrate with the `rsx` tagging syntax without becoming a fully "functional widget," it must implement `WidgetConstructor` and define its props.

```rust
trait WidgetConstructor {
  type Props;
  fn create_props(base_props: BaseProps) -> Self::Props;
  fn construct(id: Index, children: Children, props: Self::Props) -> Self;
}

/// A struct containing the base props most widgets expect.
/// The widget can choose to use them or not.
struct BaseProps {
  styles: Option<Style>,
  on_event: Option<OnEvent>
}
```

> This trait can't be used as a trait object (`dyn WidgetConstructor`), but that's okay. It's only for ensuring the methods exist and are consistent.

By having this bound on the widget struct, the `rsx` macro can then build the widget by doing something like:

```rust
let mut props = #widget_ident::create_props(base_props);
#( props.#prop_ident = #prop_value );*
let widget = #widget_ident::construct(widget_id, children, props);
```

Generated out might look something like:

```rust
// <MyWidget foo={Some(1)} bar={"Hello".to_string()} />
let mut props = MyWidget::create_props(base_props);
props.foo = Some(1);
props.bar = "Hello".to_string();
let widget = MyWidget::construct(widget_id, children, props);
```

As for the implementation of `Widget:;render`, it will look largely the same as the manual widgets. The main difference is that now props are no longer found directly on the struct.

##### Derive

One thing this allows us to *maybe* do is add a derive macro for automatically implementing `WidgetConstructor`.

```rust
#[derive(WidgetConstructor)]
struct MyWidget {
  #[widget_constructor(id)]
  widget_id: Index,
  #[widget_constructor(props = "MyProps")]
  // Or `#[widget_constructor(props)]` to assume "<Widget Name>Props"
  widget_props: MyProps,
  #[widget_constructor(children)]
  widget_children: Children,
}

#[derive(Default)]
struct MyProps {
  foo: Option<i32>,
  bar: String
}
```

A derive would be nice since in order to reduce boilerplate, but it's not necessary. And this may add an extra layer of confusion (due to required marker attributes and the `Default` bound on the props). But I left it here as a possible option and to see if there are other, better ways of implementing something like this.

<br />

<br />

#### Functional Widgets

Finally, functional widgets. These are the widgets to use when customization isn't a huge concern and you just need to define a working widget. This is probably what most people will use when defining their own widgets. Most of it is the same as the ones listed above, but I'll go over some things I'd like to suggest as well.

Consider the following widget:

```rust
#[widget]
fn MyWidget(foo: Option<i32>, bar: String) {
  // ...
}
```

From this we can generate the following code (ignoring derives, attributes, and things like that):

```rust
struct MyWidget {
  id: Index,
  props: MyWidgetProps,
  children: Children,
}

struct MyWidgetProps {
  // Base props
  styles: Option<Style>,
  on_event: Option<OnEvent>,
  // Custom props
  foo: Option<i32>,
  bar: String,
}

impl WidgetContructor for MyWidget {
  type Props = MyWidgetProps;
  // ...
}

impl Widget for MyWidget {
  // ...
}
```

All is normal so far. But something I suggested in Version 1 was to create a "Widget Context" that further separates widget logic from render logic (protecting the user from writing broken code or running into common pitfalls). This Widget Context, like `KayakContextRef`, would act as an interface for the render logic to use. It would probably be generated by a `KayakContextRef::create_context` method or something, and would look something like:

```rust
struct MyWidgetContext<'a> {
  context: &'a mut KayakContextRef,
  widget: &'a mut MyWidget,
}

impl<'a> MyWidgetContext<'a> {
  // Widget-specific logic and forwarding calls to KayakContextRef
}
```

The generated `MyWidget` would contain an associated function where all the render logic actually lives:

```rust
impl MyWidget {
  pub fn render_internal(mut context: MyWidgetContext) {
    // ...
  }
}
```

And it would be called from the actual `Widget::render` method like so:

```rust
impl Widget for MyWidget {
  // ...
  fn render(&mut self, mut context: KayakContextRef) {
    let mut context = MyWidgetContext {context: &mut context, widget: };
    Self::render_internal(context);
  }
  // ...
}
```

> Remember, this is all generated by the macro, a user would not be expected to write any of this.

<br />

---

## Issues

### Protection

One issue with this implementation is that both manual and constructed widgets still have areas that lack protection. While we can no longer run arbitrary methods from `KayakContext`, we're still able to call things like `self.set_id(Index::default())`. This leaves us protected only on one side.

However, this might be okay if we see this as A Feature™. I definitely think this is important for manual widgets, who may need full access to data on their defining struct. Even constructed widgets may want more control over their data. So if we're okay with this, then it should be fine.

### Context Differences

On this topic, it may be worth noting that the different render methods between functional widgets and the others might be confusing. The former takes a `<WidgetName>Context` and the latter takes a `KayakContext`. This may be a minor issue, but I can see some users at least initially being confused by this difference if they switch from one to the other.

And we can't (or maybe shouldn't) just make both take `KayakContext` because the render method may need immutable access to the props, ID, focusability, etc.

One way to mitigate this is just to try and mimic as much of the `KayakContext` API as possible. This shouldn't be too difficult if we impl `Deref` and `DerefMut` over it. But the smaller the API difference, the less of an issue this becomes.

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
        on_event(this, event);
      }
    }

    // Maybe something like this? (assuming it all compiles like it does in my head)
    // Although to do this, we'd need to pass `this` in as `&mut this`, which changes the API a bit
    let edits = this.edits;
    context.process_changes(edits);
  }
  // ...
}
```