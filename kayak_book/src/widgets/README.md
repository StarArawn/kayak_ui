# Widgets

A widget can be defined as any object that lives in the UI tree. Typically a widget will have some sort of visual
appearance, however that doesn't necessarily have to be the case. You might have widgets that manage state or just wrap
other widgets.

## Core Concepts

### Relationships

Widgets, by the nature of them being stored as a tree, contain certain relationships. The most common one is the *
parent-child* relationship. A widget may contain zero or more children.

<p align="center">
  <img alt="Diagram showing the parent-child relationship" src="../img/parent-child.svg" />
</p>

When a parent re-renders, it also causes its children to be re-rendered as well (more on this later).

Another implicit relationship is that of *siblings*. These aren't currently very useful in Kayak UI currently, but know
that any time two widgets share a parent, they are considered siblings.

### Props

What good is a widget if it can't hold or process data? This is where the concept of *props* comes in (short for "
properties"). Props allow data to flow from parent to child.

<p align="center">
  <img alt="Diagram showing the flow of props" src="../img/prop-flow.svg" />
</p>

For example, a widget might have access to a game's player info. It could then take the current value of the player's
health points and pass that to a widget that is specifically configured to display health points. Which brings up
another important topic when talking about widgets: modularity.

### Modularity

Widgets are meant to be modular. It's rarely a good idea to have a widget with hundreds and hundreds of lines of code.
Kayak UI instead suggests breaking parts of your UI down into individual components. This could be as simple as breaking
up logical parts of a single UI into multiple files in order to make things easier to read/maintain.

This modularity allows for widgets to be abstracted to cover more generic use-cases. Rather than having two different
widgets for a player's health points and a teammates, you could just create one reusable `Health` widget. Want to keep
the same functionality but modify the style for enemies? Wrap the `Health` widget in another widget that configures it
enough to fit your needs.

### State

To make widgets even more modular, provide greater functionality, and be reactive, widgets may contain their own state.
While props can be used to pass data to widgets, state allows data to be retained. This is important because non-state
data, such as props, are lost between renders. Without state, what goes in is all the widget has at its disposal.

This book has a [section](./state.md) dedicated to widget state so check that out for more on state!

### Lifecycle

The last core concept is that of *widget lifecycl*e. Simply put, whenever a widget's state is updated, it will be
re-rendered. This ensures that the widget is always in sync with its state. This re-render also applies to a widget's
children. It does not, however, affect its siblings.

> This is another reason why breaking up widgets is very useful: splitting state across siblings reduces the number of re-renders needed (compared to keeping all state data in the parent widget itself).

When a widget is re-rendered, it loses any non-state changes (changes to props, local variables, etc.).

## Usage

### Basic Markup

Widgets are typically defined using the angle bracket tagging system (similar to other XML-style markup languages):

```rust,noplayground
# #[widget]
# fn PlayerManager {
# rsx! {
<PlayerHud>
  <Health />
</PlayerHud>
# }
# }
#
# #[widget]
# fn PlayerHud(data: PlayerData) {
# // ...
# }
#
# #[widget]
# fn Health(hp: i32) {
# // ...
# }
#
# #[derive(Default, Debug, Clone, PartialEq)]
# struct PlayerData {
#   hp: i32
# }
```

The first widget defined in our tree is the `PlayerHud` widget. We want it to contain our `Health` widget, so we
surround that `Health` widget with our opening (`<PlayerHud>`) and closing (`</PlayerHud>`) tags.

If a widget does not contain any children, you can forgo the closing tag and just close it inline: `<Health />`

### Passing Props

Props are passed to widgets using the `prop_name={value}` syntax:

```rust,noplayground
# #[widget]
# fn PlayerManager {
# let dummy_data = PlayerData {
#   hp: 10
# };
#
# rsx! {
# <PlayerHud data={dummy_data} />
# }
# }
#
# #[widget]
# fn PlayerHud(data: PlayerData) {
#
# let current_hp = data.hp;
#
# rsx! {
  <Health hp={current_hp} />
# }
# }
#
# #[widget]
# fn Health(hp: i32) {
# // ...
# }
#
# #[derive(Default, Debug, Clone, PartialEq)]
# struct PlayerData {
#   hp: i32
# }
```

## Requirements

There are some requirements that need to be met in order for widgets to work properly. These are:

1. All props must derive or otherwise implement `Default`, `Debug`, `PartialEq`, and `Clone`.
2. All widgets must derive or otherwise implement `Default`, `Debug`, `PartialEq`, and `Clone`.