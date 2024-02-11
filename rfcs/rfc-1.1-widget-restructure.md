# RFC 1.1 - Widget Structure

This RFC is an addendum to the [original](https://github.com/StarArawn/kayak_ui/pull/56) RFC 1. The reason for this one is to cover some of the changes that RFC 1 went through during implementation that (until now) has not been documented. Additionally, I wanted to discuss a new way of adding widget contexts based on some of the discoveries made thus far.

> **Note:** This won't cover *every* change— just the ones I believe are noteworthy.

## What Changed?

### Trait Objects & Widget

One of the difficulties with the addition of `WidgetProps`, was how we would handle trait objects. Originally, the plan
was to have widgets stored as `Box<dyn Widget<Props=dyn WidgetProps>>`. This allowed us to keep things generic while still keeping the trait object. To our surprise, this compiled no problem! So we went with it. But it had a hidden
issue...

```rust
error[E0277]: the size for values of type `(dyn WidgetProps + 'static)` cannot be known at compilation time
   --> kayak_core/src/context.rs:389: 20
    |
389 | widget.render( & mut context);
    | ^ ^ ^ ^ ^ ^ doesn't have a size known at compile-time
    |
    = help: the trait `Sized` is not implemented for `(dyn WidgetProps + 'static)`
```

Attempting to _actually use_ this object resulted in a compilation error. Lesson learned: when testing, test more than
just a single use-case.

So how did we go about fixing this? By adding a new trait!

The idea comes from yew's [component source](https://github.com/yewstack/yew/blob/master/packages/yew/src/html/component/mod.rs), but essentially we introduce a new trait, `BaseWidget`, which is mostly just a copy of the `Widget` trait:

```rust
pub trait BaseWidget: SealedWidget + std::fmt::Debug + Send + Sync {
    fn constructor<P: WidgetProps>(props: P) -> Self where Self: Sized;
    fn get_id(&self) -> Index;
    fn set_id(&mut self, id: Index);
    fn get_props(&self) -> &dyn WidgetProps;
    fn get_props_mut(&mut self) -> &mut dyn WidgetProps;
    fn render(&mut self, context: &mut KayakContextRef);
    fn get_name(&self) -> &'static str;
    fn on_event(&mut self, context: &mut KayakContext, event: &mut Event);
}
```

> The `SealedWidget` trait is a private trait that prevents outside crates from implementing `BaseWidget` directly

All implementors of `Widget` also implement `BaseWidget` via a blanket implementation, allowing them to be used
interchangeably.

More importantly, it allows us to hide the associated `Props` type. This means that we can replace our original trait
object with this one: `Box<dyn BaseWidget>`. And *poof!* error gone.

An awesome side-effect of this is it allows us to also enforce some type bounds that used to be implicit. Specifically,
we can now enforce the `Clone`, `Default`, and `PartialEq` bounds that were required but not by the `Widget` trait
itself. So now we can have trait bounds like this:

```rust
pub trait Widget: std::fmt::Debug + Clone + Default + PartialEq + AsAny + Send + Sync {
    // This also includes the associated Props type!
    type Props: WidgetProps + Clone + Default + PartialEq;

    // ...
}
```

> We can't do the same for the `WidgetProps` trait since it's still used as a trait object on `BaseWidget`.

### `WidgetProps`

#### Children

The `WidgetProps` trait is great in that it both cleans up a lot of internal code and reduces in the user-facing *magic*
🪄. It does this by allowing a user to specify which props a widget accepts by defining a struct:

```rust
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct MyWidgetProps {
    /// Exposed to callers outside this module
    pub value: i32
    /// Hidden to outside modules
    internal_value: i32,
    /// Specifies that this widget accepts styles in a prop called "my_styles"
    #[prop_field(Styles)]
    pub my_styles: Option<Style>
    /// Specifies that this widget accepts children in a prop called "my_children"
    #[prop_field(Children)]
    pub my_children: Option<Children>
}
```

But there's an issue with this struct that the original RFC missed. See, we can call props by whatever we want since
the `rsx!` macro simply takes a prop's name and applies it to the widget's props.

So something like:

```rust
rsx! {
  <MyWidget value={10 + 4} foo={"I do not exist".to_string()} />
}
```

Results in the generated output:

```rust
// ...
props.value = 10 + 4;
props.foo = "I do not exist".to_string(); // ERROR! Prop "foo" does not exist!
// ...
```

This works great for defining the props because we don't need to know which ones are actually accepted, thanks to the
compiler. But it will always fail for one case: children.

Since `children` is actually defined by the content between opening and closing widget tags, we can't take deduce its
identifier. So in reality, the `#[prop_field(Children)]` attribute did nothing if the field wasn't named `children`. And
passing children to this widget would result in a compiler error.

The solution to this was to add a method to `WidgetProps` to allow us to set the children dynamically:

```rust
pub trait WidgetProps: std::fmt::Debug + AsAny + Send + Sync {
    // ...
    fn set_children(&mut self, children: Option<Children>);
}
```

#### Naming

Another small change from RFC 1 is the required naming system. It was thought props would need to use a name in the form of `<Widget Name>Props` in order for us to actually create a widget's props in the `rsx!` macro (remember, we don't have great access to type information when processing macros).

As it turns out, though, we don't need this requirement due to the changes `Widget` mentioned earlier. Instead, we can
generate the props like so:

```rust
let mut props = < # name as # kayak_core::Widget>::Props::default ();
// Which translates to something like:
// let mut props = <MyWidget as kayak_ui::core::Widget>::Props::default();
```

From there, we just assign the props individually, line by line. Easy!

#### Common Prop Types

All common props now have more consistent types. Namely, `Children` has been converted to a proper struct rather than a type alias. It also no longer wraps `Option` but is instead wrapped *by* `Option` so as to be more like the other common props.

Additionally, both `OnEvent` and `Children` now implement `Debug`, `Clone`, and `PartialEq` (the latter two essentially
do nothing). This means we no longer need to use the [derivative](https://crates.io/crates/derivative) crate to ignore those fields, thus reducing the verbosity and hidden gotchas of creating a prop struct.

For reference, this is what we used to have to do:

```rust
#[derive(WidgetProps, Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
pub struct MyWidgetProps {
    some_value: i32,
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    #[prop_field(Children)]
    pub children: Option<Children>
}
```

### TL;DR

1. Added `BaseWidget` trait
2. Enforced `Clone`, `Default`, and `PartialEq` bounds on `Widget` and `Widget::Props`
3. Added `WidgetProps::set_children(...)` trait method
4. Removed props naming requirement
5. Removed the need for the [derivative](https://crates.io/crates/derivative) crate

## Widget Context

With [#68](https://github.com/StarArawn/kayak_ui/pull/68) and [#69](https://github.com/StarArawn/kayak_ui/pull/69) just merged, the only big change from RFC 1 left is the widget context.

The RFC suggested that this be generated by the `#[widget]` macro. The reason for this was so that we could store a
direct reference to the widget on the generated struct:

```rust
struct MyWidgetContext<'a> {
    context: &'a mut KayakContextRef,
    widget: &'a mut MyWidget,
}
```

However, RFC 1.1 suggests a different way of doing this.

Instead of generating a struct for every widget, we can specify a generic `WidgetContext` struct. This struct would look
more like:

```rust
struct WidgetContext<'a, TProps: WidgetProps> {
    context: &'a mut KayakContextRef,
    props: &'a mut TProps,
    id: Index,
}
```

But why? What benefits do we gain by doing this instead of the original?

Well for one, it reduces the amount of code we actually need to generate. But beyond that it significantly reduces the *
magic* 🪄 involved in defining a widget. The idea was just to replace the hidden `KayakContextRef` with a hidden widget context— not great.

By using `WidgetContext` our functional widget actually works more like a regular function now:

```rust
fn MyWidget(context: WidgetContext<MyWidgetProps>) {
    // ...
}
```

No more hidden `context`. No more hidden `self`. What's available to the user is defined by *them* very plainly.

#### Prop Access

This also makes prop access much more intuitive. By calling something like `context.props()` we can return an immutable reference to the widget's props. This fixes two minor issues in the current system.

The first is implicit cloning. Right now the `rsx!` macro inserts a clone on the props. However, this may be a costly
operation for some prop values and could simply be avoided by just letting the user clone what they need.

The second problem this solves is the (in my opinion) anti-pattern of mutating props. Props should be purely input data. It doesn't make sense that we can change or mutate this input when it's just going to change on the next render.

This feels wrong:

```rust
fn MyWidget(props: MyWidgetProps) {
    let my_styles = Some(Style { /* ... */ });
    props.style = my_styles;
    // ...
}
```

Instead, it makes more sense to do this:

```rust
fn MyWidget(context: WidgetContext<MyWidgetProps>) {
    let my_styles = Some(Style { /* ... */ });
    context.set_styles(my_styles);
    // ...
}
```

This might help avoid confusion wherein users think they can change other non-common props without realizing that those changes will not stick across renders.

And note that either way works. In fact, `WidgetContext::set_styles(...)` would internally just do something like in the first code snippet. That's just the nature of how we store styles, event handlers, etc. However, the important thing is
that it doesn't leave users thinking they can or should mutate props.

> This is all leans more on the subjective side, so perhaps I'm wrong here and we should just allow calling `WidgetContext::props_mut()`.

#### Naming Requirement

The last bit of hidden code we have left (at least one of the more important ones) is the naming of `context`. Ideally,
a user could call this whatever they want (such as `ctx`, `this`, or `self`). But we need it to be called `context` when
we get to the `rsx!` macro. This also means we can't shadow the `context` identifier within the function body.

Unfortunately, there's no easy way of getting around this. The best we *could* do is create a variable that no one would
ever collide with (unintentionally at least):

```rust
let kayak_ui_widget_context = context;
// ...
```

However, this won't work due to `context` being a mutable reference.

So for now, the naming requirement remains (and should be enforced by the `#[widget]` macro).

