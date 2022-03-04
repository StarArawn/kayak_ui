# RFC 4 - Text Selection Design

The purpose of this RFC is to discuss the design and implementations of text selection, possible considerations, and challenges.

## Goals

**Main Goals**

* Visualize a cursor within focused text fields
* Allow text within widgets to be selectable
* Allow text across widgets to be selectable

**Secondary Goals**

* All of the above with future accessibility in mind

## Motivation

The primary goal of this RFC is to actually render a cursor within the `TextBox` widget. And while we could come up with a specific solution to that problem, I think it would be better to look at text selection as a whole first. This is because rendering a cursor and moving it around is very similar to the broader concept of text selection: a text cursor is really just a selection of zero width.

We can explore text cursors with more specificity in another RFC/PR. For now, this one will cover the general topics and considerations for text selection, which should (hopefully) aid an actual design and implementation for text cursors.

## Additional Context

### Resources

A lot of the content of this RFC is taken from the DOM Standard [specs](https://dom.spec.whatwg.org/). Of course, it's been paraphrased and translated to fit the needs of Kayak. However, it's still great resource for understanding how the web has chosen to handle things.

### Discussion

Parts of this RFC require discussion before a consensus is reached on how to tackle a certain problem. Anything is up for discussion really, but the parts that really need it are marked with the `💬` emoji.

## Terminology

### Character

Unless otherwise stated, a <em>**character**</em> is a blanket term referring to a *variable-width* character. This means that a character is not necessarily a single byte and could be made up of one or more, as is often the case when working with encodings such as [UTF-8](https://en.wikipedia.org/wiki/UTF-8). 

It's important to account for this because it better prepares us for multi-language support.

Additionally, when it comes to text selection, we generally want the [*grapheme cluster*](https://unicode.org/reports/tr29/#Grapheme_Cluster_Boundaries). This is what—to a user— is typically viewed as a "character".

> Rust's built-in [`chars()`](https://doc.rust-lang.org/stable/std/primitive.str.html#method.chars) method breaks strings down into *Unicode Scalar Values*. For segmentation along grapheme clusters, an external crate like [`unicode-segmentation`](https://crates.io/crates/unicode-segmentation) will likely need to be used.

### Range

For the purposes of text selection, a <em>**range**</em> is a set of two positions: <em>**start**</em> and <em>**end**</em>. These two points mark the bounds of the selected text. Furthermore, the start and end points are not required to be in any particular order. The start point can come first or it can come after the end. They can even be the same point (known as a "collapsed" or "empty" range).

## Design

### 1. Retrieving Widget Info

#### 1.1. Text Content

Before we start discussing how we select text and all that, we need to discuss how we even process our widgets textually. Thankfully we already have a system in place to determine if a widget displays text and what text content it displays. All we need to do is check if a widget has a `RenderCommand::Text` render command.

#### 1.2. Length

Another important bit of information we may need is the the *length* of a widget. The length of a widget is determined as the following:

* If the widget contains text, return the number of characters[^1] in the text
* If the widget has children, return the number of children
* Otherwise, return 0

This information will be used to verify that a range is valid.

#### 1.3. `Node` Methods

Most of this data will be available via the `Node` object (since all styles and other data are resolved into nodes). As such, we'll add a few convenience methods to access the required widget info:

```rust
impl Node {
  /// Get the text contents of this node, if any, otherwise `None`
  pub fn content(&self) -> Option<&str> {/* ... */}
  /// Get the content length of this node
  pub fn len(&self) -> usize {/* ... */}
  /// Checks whether this is a text node or not
  pub fn is_text(&self) -> bool {/* ... */}
  // etc.
}
```

> The `&str` in the code above may need to just be `String` depending on implementation. Either way, the general idea of it will remain the same.

### 2. Defining the Range

#### 2.1. Bounds

Since a [range](#range) is just a start and an end point, we first need a way of identifying those points. The big questions we need to ask are:

* *What node does this position belong to?*
* *What character[^1] is this position pointing to?*

As long as we have the answer to those two questions we'll know exactly where the position is, and consequently, how our range is defined. Luckily, both are really simple to represent: all we need is a widget's `id` and the character[^1] index from the start of the widget's text (i.e. the number of captured characters).

> This can also be used for non-text nodes as well. In that case, the offset would just refer to the number of captured *children* rather than characters.

This can succinctly be stored in a struct like:

```rust
// DEMONSTRATION - This may change based on implementation
pub struct RangeBound {
  node: Index,
  offset: usize
}
```

> In examples throughout this RFC, we'll use the short-form notation of `(Node, Offset)`. So a bound matching the node of ID 2 at offset 3 would be notated as (2, 3).

Given any two `RangeBound` objects, we can define an actual range like so:

```rust
// DEMONSTRATION - This may change based on implementation
pub struct Range {
  start: RangeBound,
  end: RangeBound
}
```

If we allow ourselves to jump ahead for a moment, we might wonder if text selection will always have a start and end. Could one exist without the other? It might be possible that we only know the start or end of a range and not both. However, recall that a range is perfectly valid even if its start and end are the same point. Therefore, if we can only define the range with a single point, we can simply set both the start *and* end to that point.

##### 2.1.1 `RangeBound` Methods

```rust
impl RangeBound {
  /// Get the ID of the widget
  pub fn id(&self) -> Index {/* ... */}
  /// Get the offset within the widget
  pub fn offset(&self) -> usize {/* ... */}
  // etc.
}
```

##### 2.1.2 `Range` Methods

```rust
impl Range {
  /// Get the start bound
  pub fn start(&self) -> RangeBound {/* ... */}
  /// Set the start bound
  pub fn set_start(&mut self, start: RangeBound) {/* ... */}
  /// Get the end bound
  pub fn end(&self) -> RangeBound {/* ... */}
  /// Set the end bound
  pub fn set_end(&mut self, end: RangeBound) {/* ... */}
  /// Collapse and move both start and end bounds to the given position
  pub fn move(&mut self, node: Index, offset: usize) {/* ... */}
  /// Check if the range is collapsed (start is the same as end)
  pub fn is_collapsed(&self) -> bool {/* ... */}
  /// Collapse the end bound to the start point
  pub fn collapse_to_start(&mut self) {/* ... */}
  /// Collapse the start bound to the end point
  pub fn collapse_to_end(&mut self) {/* ... */}
  // etc.
}
```

#### 2.2. The In-Between

With the *bounds* of the range defined, how do we actually extract the content? There are two scenarios:

1. The start and end bounds lie within the same widget
2. The start and end bound lie within different widgets

Obviously, getting the string from the first scenario is trivial: just snag the characters[^1] from start to end.

The second one, however, forces us to cross node boundaries in order to piece together the string. But how do we get the *in-between* content? Firstly, what content do we even want?

###### Example 2.2.1

Let's say we have a widget tree like this:

```
            0
         ┌──────┐
         │ Root │
         └──┬───┘
            │
    1┌──────┴───────┐2
     │              │
  ┌──▼───┐     ┌────▼────┐
  │ Text │     │ Element │
  └──┬───┘     └────┬────┘
     │              │
     │        3┌────┴─────┐4
     │         │          │
┌────▼────┐ ┌──▼───┐ ┌────▼────┐
│ "Hello" │ │ Text │ │ Element │
└─────────┘ └──┬───┘ └────┬────┘
               │          │
               │          │5
               │          │
            ┌──▼───┐   ┌──▼───┐
            │ "Wo" │   │ Text │
            └──────┘   └──┬───┘
                          │
                          │
                          │
                      ┌───▼────┐
                      │ "rld!" │
                      └────────┘
```

> Text contents are displayed as nodes for the purposes of demonstration, but are not actually viewed as nodes within Kayak. This is also why they are not given IDs in this example.

Let's say our start bound is (1, 0) and our end bound is (5, 3). In order to get the full string we need to traverse our widget tree. So we will traverse our tree [**depth-first**](https://dom.spec.whatwg.org/#concept-tree-order), starting with our start node and ending with the end node.

```
Node 1 - Some("Hello") | Total - "Hello"
Node 2 - None          | Total - "Hello"
Node 3 - Some("Wo")    | Total - "HelloWo"
Node 4 - None          | Total - "HelloWo"
Node 5 - Some("rld")   | Total - "HelloWorld"
```

Doing this, we end up with our desired string which is `HelloWorld` (exclamation point left out since it's at offset 4).

This is pretty much the simplest way of collecting the string of characters our range covers. However, traversing a tree every time we want to grab content from our range is not super efficient. Instead, as long as the range is the same and the widgets are unchanged, we can just return a cached value of the substring. There are other ways to optimize and handle caching (such reusing in-between content if a new range overlaps the previous one), but this is by far the simplest.

> It's important to also make note that ranges should *not* collect text content automatically. Doing so could result in loads of unnecessary tree traversals. Instead, we should only calculate this data the moment it's actually needed.

#### 2.3. Non-Text Nodes

Ranges can stretch across non-text nodes (nodes without text content) as well. This might seem pointless, but it's very fundamental to text selection. You can try it out by going to any webpage then click-and-drag the mouse from any non-text element. You'll notice that doing this typically captures whole chunks of text rather than just individual characters. Let's briefly look at why that is.

###### Example 2.3.1

Take this widget tree:

```
           0
         ┌──────┐
         │ Root │
         └──┬───┘
            │
    1┌──────┴───────┐2
     │              │
┌────▼────┐    ┌────▼────┐
│ Element │    │ Element │
└────┬────┘    └────┬────┘
     │              │
    3│        4┌────┴────┐5
     │         │         │
  ┌──▼───┐  ┌──▼───┐  ┌──▼───┐
  │ Text │  │ Text │  │ Text │
  └──┬───┘  └──┬───┘  └──┬───┘
     │         │         │
     │         │         │
     │         │         │
 ┌───▼───┐ ┌───▼───┐ ┌───▼───┐
 │ "Foo" │ │ "Bar" │ │ "Baz" │
 └───────┘ └───────┘ └───────┘
```

Let's say we start our selection at the root— a non-text node— and offset 0, giving us a boundary point of (0, 0). Now let's end the range at (0, 1). With the range defined, we can now piece together our selected content:

```
Node 0 - None        | Total - ""
Node 1 - None        | Total - ""
Node 3 - Some("Foo") | Total - "Foo"
```

Because Node 0 has no text content, its offset refers to children rather than characters. Because of this, ranging from an offset of 0 to an offset of 1 captures the entirety of the first child. And this captures the entire string, `"Foo"`.

> In fact, we've been doing this already. A node that's fully captured by a range can be seen as sub-range from (Node, 0) to (Node, Length-1). And we can apply the same rules to the children as well, which is why we capture *all* of Node 3 in the example above. This is pretty much what happens when a range spans across a full node, such as in [Example 2.2.1](#example-221).

If instead we ended the range at (4, 1), we'd get the following traversal:

```
Node 0 - None        | Total - ""
Node 1 - None        | Total - ""
Node 3 - Some("Foo") | Total - "Foo"
Node 2 - None        | Total - "Foo"
Node 4 - Some("B")   | Total - "FooB"
```

And if we set the range to go from (0, 1) to (0, 2), we'd get:

```
Node 0 - None          | Total - ""
Node 2 - None          | Total - ""
Node 4 - Some("Bar")   | Total - "Bar"
Node 5 - Some("Baz")   | Total - "BarBaz"
```

As you can see, creating ranges across non-text nodes is useful for quickly getting an entire subtree of content. In the above case, simply spanning from (0, 1) to (0, 2) captured the entirety of Node 2's subtree.

#### 2.4. Dynamic vs Static

The DOM standard makes a [distinction](https://dom.spec.whatwg.org/#introduction-to-dom-ranges) between dynamic (they use the term "live") and static ranges. Currently, what we have described thus far is akin to [*static* ranges](https://dom.spec.whatwg.org/#interface-staticrange). This because we don't perform any checks or diffing to ensure that the content of a range remains the same. It would be possible to do so— and it would be worth exploring in the future. But for now, we can consider dynamic ranges out of scope for our purposes.

### 3. Positioning

One of the biggest challenges text selection faces is determining the actual boundary points. The basic idea is: *click here, set boundary point(s)*. We can pretty easily get the clicked node, but how do we determine the offset of the boundary?

There are two cases to consider: non-text nodes and text nodes.

#### 3.1. Non-Text Nodes

This is actually the simplest of the three options since we have ready access to the layout of widgets. We can get the offset by doing the following:

1. If *widget* has no children, return 0
2. Iterate through each *child*:
   1. If *point* is below *child*, continue
   2. If *point* is to the right of *child*, continue
   3. Otherwise, return *child*'s index

#### 3.2. Text Nodes

Getting the offset on a text node is slightly trickier. It involves similar steps for non-text nodes, but requires a bit more work since the individual characters[^1] are not themselves nodes. To find the correct character offset, we have to do some calculations. There's a few different ways we could choose to do this, so I will describe each.

> Some of these methods describe caching and other mechanisms. The actual implementation is not super important at the current stage of this RFC. After some discussion over the best approach, we can then narrow in on implementation details.

> Additionally, all methods need to account for various text layouts. This includes text alignment like center and right aligned, as well as language-specific layouts such as right-to-left (RTL) or vertical based languages. These are things we should definitely keep at the front of our minds when choosing a method and designing its implementation.

##### 3.2.1. Brute Force Method

This is the simplest method. Basically, given a point we:

1. Start at character[^1] *index* 0
2. Calculate the *bounding box* of the character at *index*
3. If *bounding box* contains *point*, return *index*
4. If *point* is to the left or above *bounding box*, return *index*
5. Increment *index*
6. If *index* is less than *content length*, go to #2
7. Otherwise, return *content length*

Since we don't know the position of characters in advance, we have to calculate each individually in order to lay them out. And since we have to do this sequentially, we might as well perform the check right after the calculation.

This method should be easy to implement but is not great when it comes to performance. Imagine a long paragraph. With the worst case being the last character is the one clicked, we would need to calculate each character in the paragraph— every time! 

In other words, this is `O(n)` at worst.

Obviously, we can do a bit better.

##### 3.2.2. Line Method

One way we can get around brute forcing every possible character (worst case), we can instead implement line-by-line caching. When we first calculate the text layout, we can store the index of the character that starts a given line. For example:

```
The quick brown fox jumped\n
over the lazy dog.
```

We know the starting character of both lines and can save their indexes. This is 0 for the 'T' in "The" and 27 for the 'o' in "over" (includes spaces and newlines).

Now when we receive a click, we can calculate which line it belongs to. With the line determined, we then perform the brute-force method like before. However, rather than possibly calculating every character we'll only ever calculate the characters in a given line. As long as we're not dealing with massively long lines, this should be a lot faster than the brute force method alone.

This results in a time complexity of `O(c)`, where `c` is the number of characters in the line with the most characters.

It is also more memory-efficient than other methods. It only needs to cache a single `usize` for each line, requiring only a `Vec<usize>` in total.

##### 3.2.3. Word Box Method

Another caching strategy involves storing the bounding box of each word[^2] (relative to the text's layout). By storing the bounding box for each word along with the total character[^1] length at that word, we can perform a binary search to find the actual containing word, followed by the brute-force method over the individual characters.

Let's pretend we clicked the 'b' in "brown" in the following example:

```
The quick brown fox jumped\n
over the lazy dog.
```

The word "brown" might have the following data cached:

```rust
WordBox {
  x: 34.3,
  y: 0.0,
  width: 28.2,
  height: 14.1,
  chars_before: 10,
}
```

We can now perform the search:

1. Create indexes *start*, *end*, and *current*
2. Set *current* to mid-point between *start* and *end*
3. Get *bounding box* at *current*
4. If *point* is to the left or above *bounding box*, set *end* to *current* and go to #2
5. If *point* is to the right or below *bounding box*, set *start* to *current* and go to #2
6. If *point* is contained by *bounding box*, normalize *point* relative to *bounding box*
7. Perform brute force method on characters in *word* at *current* to find *offset*
8. Increment *offset* by *chars_before*
9. Return *offset*

This results in us doing the following checks:

```
start: 0 | end: 17 | current: 8
" " - before
start: 0 | end: 8 | current: 4
" " - after
start: 4 | end: 8 | current: 6
" " - before
start: 4 | end: 6 | current: 5
"brown" - before
Perform brute force...
Offset: 1 | Chars Before: 10
Result: 11
```

This is significantly faster, resulting in a worst-case time complexity of `O(log(n) + m)` where `n` is the number of words[^2] and `m` is the character[^1] length of the longest word.

But, while this method is fast, it does come with higher memory overhead. Each word needs to store their `x`, `y`, `width`, `height`, and `chars_before` in order to be optimal. This is four `f32`s and one `usize`, resulting a total size of around 24 bytes[^3] per word. This may be more than we're willing to give. Or maybe it's perfectly reasonable given its speed boost.

##### 3.2.4. Comparison and Discussion

Here's a quick rundown of each method using the example sentence:

```
The quick brown fox jumped\n
over the lazy dog.
```

Assume we click on the 'a' in "lazy".

| Method      | # of Checks | Cached Size[^3][^4] |
| ----------- | ----------- | ------------------- |
| Brute Force | 38          | -                   |
| Line        | 12          | 16 bytes            |
| WordBox     | 6           | 408 bytes           |

Obviously, this is only a single example and can't show all the edge cases and averaged results. However, it does give us a quick look into what each method might look like.

> 💬 Which method should we use to calculate character offset? Are there alternatives? And are there any challenges that haven't been considered?

### 4. Selection

With all the setup out of the way, we can now look into actually handling text selection. Again, I'm not going to dive into the implementation details too much. This is meant to just give a broad overview of text selection.

#### 4.1. The Selection API

At its core, a selection is just a fancy range. One thing it does differently from a normal range, however, is keep track of an *anchor* and a *head* (the web uses the term ["focus,"](https://developer.mozilla.org/en-US/docs/Web/API/Selection#focus_of_a_selection) but I'd like to avoid that term in order to prevent confusion with widget focus). 

The *anchor* is the part of the selection that doesn't change while the *head* is the part that can be moved. When clicking and dragging, for example, the initial click sets the anchor while the drag moves the head. While it doesn't matter which bound is which (start or end), we may as well follow the web conventions:

* If the selection direction is *forwards*— the anchor comes before the head— we set the anchor as the start and the head as the end
* If the selection direction is *backwards*— the anchor comes after the head— we set the anchor as the end and the head as the start
* If the selection direction is *directionless*— the anchor and the head are the same (a collapsed range)— it doesn't matter which is set to which

Additionally, the selection should contain the *common ancestor* of the two bounds. This is the deepest node in the widget tree that contains both bounds and is vital for efficiently processing certain calculations. This will also be very helpful when we add [dynamic ranges](#24-dynamic-vs-static) since it can help determine if a tree mutation possibly affected the selected contents.

##### 4.1.1. `Selection` Methods

Here is a list of potential methods we may want to include on the selection object (not exhaustive):

```rust
impl Selection {
  /// Get the anchor
  pub fn anchor(&self) -> RangeBound {/* ... */}
  /// Set the anchor
  pub fn set_anchor(&mut self, id: Index, offset: usize) {/* ... */}
  /// Get the head bound
  pub fn head(&self) -> RangeBound {/* ... */}
  /// Set the head
  pub fn set_head(&mut self, id: Index, offset: usize) {/* ... */}
  /// Get the range
  pub fn range(&self) -> Range {/* ... */}
  /// Set the range
  pub fn set_range(&mut self, range: Range) {/* ... */}
  /// Shift the offset by the given amount
  pub fn shift_offset(&mut self, offset: isize) {/* ... */}
  // etc.
}
```

> We'll also likely want to expose some methods on `Range` in `Selection` for convenience.

> If [access to the widget tree](#413-alternative---arc-ing) is granted, we can include additional methods here such as `contents()` and `common_ancestor()`.

##### 4.1.2. Interfacing with `KayakContext`

One big issue the API will need to address is handling retrieving and mutating the selection. The reason this is difficult is because we need access to not only the widget tree, but the selection itself. Since we might mutate the selection or the tree, we need mutable access to both. This is obviously a challenge when it comes to Rust's borrow rules.

To solve this, I think we need to diverge from web tech a bit. Whereas in Javascript, the selection object can directly access and mutate data in the document, our selection object should probably just be used to define *how* to access or mutate data. This means we use the selection object as a parameter to those methods as opposed to actually calling the methods on the selection object directly.

So in essence, this would look something like:

```rust
// Get a clone of the current selection
let mut selection = context.selection();
// Mutate it
selection.shift_offset(-1);
// Use it to get content
let content = context.contents(selection.range());
// Set the new selection
context.set_selection(selection);
```

Here, we see that the selection object on its own does nothing. It's purely data meant to be used by `KayakContext`.

At the cost of being slightly more verbose, we now don't have to concern ourselves too much with borrow checking. This does, however, leave us with one possibly annoying issue: bounds checking.

Since we don't have access to the widget tree, we are forced to accept any and every range, even if it's not possible. We can't throw an error on this code, for example:

```rust
let mut selection = context.selection();

// Assume the node our head is at only has a length of 10
// This should, therefore, not be possible:
selection.shift_offset(100000000);
```

Even though we know the offset can't extend beyond the length of the content, it's still allowed to. Why? Because we can't verify that this is wrong without a reference to the widget tree. Therefore, the actual error needs to be thrown when we try to use it:

```rust
context.set_selection(selection); // "Error: Offset exceeds node's length of 10"
```

This isn't the worst, but it may result in confusion and difficulty on the user's end.

##### 4.1.3. Alternative - `Arc`-ing

Selection only deals with two things: itself and the node tree. As we saw in the section above, we can freely edit a selection, but need to use `KayakContext` as a bridge to read/write the tree. This was due to issues with borrowing while we mutably borrow `KayakContext` for rendering.

One way around this would be to either change `KayakContext::widget_manager` from just a basic `WidgetManager` to an `Arc<RwLock<WidgetManager>>` (if we store the selection object in `KayakContext`) or we change `WidgetManager::node_tree` to an `Arc<RwLock<Tree>>`. Doing so, allows us to store a reference to the manager— or tree— in the selection object directly.

It should be safe to do so since we render widgets one-at-a-time on a single thread anyway. And this would allow the API to look something more like:

```rust
// (NO CHANGE) Get a clone of the current selection
let mut selection = context.selection();
// Mutate it (now with bounds checking)
selection.shift_offset(-1).unwrap();
// Use it to get content (now done via the selection itself)
let content = selection.contents();
// (NO CHANGE) Set the new selection
context.set_selection(selection);
```

It's not a major change but certainly an improvement.

However, this could be a large refactor and something we'd want to really consider before doing. It may be beneficial to do something like this in the long run for other systems, but it might also cause unforeseen issues.

> 💬 Should a major refactor like this be done? What are the possible issues this might create? Is it worth it? Should we do it to only the node tree or the manager itself?

##### 4.1.4. `KayakContext` Methods

> By extension these methods also apply to `KayakContextRef` (for all user-facing APIs)

```rust
impl KayakContext {
  /// Get the current selection (if nothing is selected, the range should be "collapsed")
  pub fn selection(&self) -> Selection {/* ... */}
  /// Set the current selection
  pub fn set_selection(&mut self, selection: Selection) -> Result<(), WidgetRangeError> {/* ... */}
  /// Get the string content with the given range
  pub fn contents(&self, range: Range) -> Result<&str, WidgetRangeError> {/* ... */}
  /// Get the common ancestor of the given range
  pub fn common_ancestor(&self, range: Range) -> Index {/* ... */}
}
```

> Note that whether we go with the [alternative](#413-alternative---arc-ing) design or not, we likely still want methods like `contents(...)` on `KayakContext` so that they can be used outside of widgets and apart from the physical selection. However, things like `common_ancestor(...)` can be moved to `Selection`.

##### 4.1.5. Ownership

One quick note to make is that the selection object should likely be stored in `WidgetManager`. This might come down to actual implementation, but it's probably best to keep it there so processing things like content and validating bounds can be done at any point in time— without having to pass the selection object in as a parameter from `KayakContext`.

However, this depends on whether we want the selection object to store a reference to the node tree or the entire `WidgetManager` (see section [4.1.3 Alternative - `Arc`-ing](#413-alternative---arc-ing) for details).

##### 4.1.6. Validating Selection

It's important that our selection always remain valid. In other words, our selection's bounds must always point to an existing widget and a valid offset within that widget. Again, without proper support for [dynamic ranges](#24-dynamic-vs-static) we can't really diff our widgets to see if their content is the same and account for changes. However, we *can* ensure the bounds are always valid.

If one of the selection's bounds has been re-rendered, we can collapse the selection to the root widget. In this way, we ensure that no matter what happened to the bound widget, we don't ever have an invalid selection.

> We might need to verify this on each render since we don't have widget removal detection available yet.

We could potentially only collapse to root if the offset of the re-rendered widget is invalid, however, this might cause some discrepancies. Consider the following:

```
Node 1: "Hello "
Node 2: "World!"
```

Say we select from the 'e' in "Hello" and the 'r' in "World". If we only collapse on invalid offsets, we could potentially change Node 1 to this without collapsing:

```
Node 1: "Hello everyone in this lovely "
Node 2: "World!"
```

This seems reasonable since it's still between the bounds. But does this next example make sense?

```
Node 1: "Hello "
Node 2: "everyone in this lovely World!"
```

Now our selection's content is "Hello eve". The offset in Node 2 is still valid, it just points to the wrong character.

To avoid confusion, it's probably better to just have the rule be: if the widget is re-rendered, the selection is collapsed to the root.

#### 4.2. Creating the Selection

The [Selection API](#41-the-selection-api) is useful for allowing users (and ourselves) to manually control the selection. However, we don't want them to have to do this all manually. It would be obviously be better to do most of the basic stuff automatically for them. In order to do this, we'll need to augment our event system to include a few more default actions and events.

Firstly, what causes a selection to be made or augmented? Here are the ways we should allow this:

* Double-click to select a word
* Triple-click to select an entire node
* Click and drag to select a custom range

We may also want to include other niceties, although they may be more difficult to implement:

* Use <kbd>Shift</kbd>+<kbd>→</kbd> or <kbd>Shift</kbd>+<kbd>←</kbd> to expand/shrink the selection
  * Hold <kbd>Alt</kbd> to shift by a whole word
  * Hold <kbd>⌘</kbd> or <kbd>Win</kbd> to expand/shrink the selection to the start/end of the current line (although, I'm not sure if this actually the behavior on Windows)
* Use <kbd>Shift</kbd>+<kbd>↑</kbd> or <kbd>Shift</kbd>+<kbd>↓</kbd> to expand/shrink the selection up or down a line

#### 4.2.1. Multi-Clicks

Detecting double-click and triple-click should be relatively simple. We just need to store the location and `Instant` for up to three clicks and check that they're within a certain range. We may even want to expose an event for these (or at least for double-click):

```rust
enum EventType {
  // ...
  DoubleClick(CursorEvent),
  // ...
}
```

With that done, we can add a default action to be performed on each.

> It's important to note that these "default actions" should not be preventable by `event.prevent_default()`. Allowing that would create inconsistencies for how selection can be created (this wouldn't stop click-and-drag, for example).

For the double-click, we can do the following:

1. Get *position* using one of the previously discussed [methods](#3-positioning)
2. If non-text node, collapse to *position* and return
3. Otherwise, get node *content*
4. Use *offset* to identify *word*
5. Set *anchor* to start offset of *word* and set *head* to end offset of word

For triple-click, things may change depending on the selected position-detection method. Since we need to access lines, directly, this may be a reason to use the [line method](#322-line-method). Otherwise, some other method of calculating the desired line may be needed.

If we use the line method, though, we get something like:

1. Get *position*
2. If non-text node, collapse to *position* and return
3. Otherwise, get *line offset* and *line index*
4. Set *anchor* to *line offset* and set *head* to offset of the line at *line index* + 1

#### 4.2.2. Click-and-Drag

Click and drag is actually relatively straightforward. Anytime we click, we collapse the selection to the range bound at that location. If we hold the cursor down, we'll continue to update the head of the selection every time the mouse moves.  

Additionally, if we hold <kbd>Shift</kbd> and click elsewhere, we'll do the following:

1. Get the boundary, *A*, of the anchor and the boundary, *B*, of the head
2. Get the boundary, *C*, of the clicked location
3. If *C* is closer to *A* than to *B*, set anchor to *B* and head to *C*
4. If *C* is closer to *B* than to *A*, set anchor to *A* and head to *C*

> If *C* is between *A* and *B*, we will likely need to traverse the tree to determine which one is closer, unless a better solution is found

#### 4.2.3. Expanding/Shrinking Selection

We should also consider making it possible to expand/shrink the selection using arrow keys. This feature should only be possible if there is currently a *non-collapsed* selection and the user is also holding down the <kbd>Shift</kbd> key.

Pressing <kbd>←</kbd>, <kbd>→</kbd>, <kbd>↑</kbd>, or <kbd>↓</kbd> should work like so:

1. If *selection* is collapsed, return
2. Get arrow key direction, *D*
3. If *selection* has not yet been moved, do the following:
   1. If *D* is pointing left or up, set the rightmost bound as the *anchor*
   2. Otherwise, set leftmost bound as the *anchor*
   3. Save current offset of *head*, *H*, from start offset of current line
4. If *D* is moving left or right:
   1. Shift *head* by an offset of 1 in that direction
   2. Save current offset of *head*, *H*, from start offset of current line
5. If *D* is moving up or down:
   1. Get line, *L*, in direction of *D*
   2. Get start offset of *L*, *S*
   3. Get content length of *L*, *C*
   4. Set offset of *head* to *min( S + H , C )*
6. If *head* crosses over *anchor*, collapse range

#### 4.2.4. Selection Events

Selection is its own thing and should *never* be cancelable from some other event. Calling `event.prevent_default()` on  `EventType::Click` should not affect selection. Instead, we will dispatch selection events that will allow users to respond to and prevent selection.

```rust
enum EventType {
  // ...
  SelectionStart(SelectionEvent),
  SelectionEnd(SelectionEvent),
  // ...
}
```

###### `SelectionStart` Event

The `SelectionStart` event will be invoked whenever a new selection is started in the target. It should be able to propagate with its target being the node at the selection's anchor. If canceled, the current selection should not change and the new selection should not be created.

###### `SelectionEnd` Event

The `SelectionEnd` event will be invoked whenever a selection is ended in the target. It should be able to propagate with its target being the node at the selection's head. If canceled, the current selection should not change and the head should not be moved.

This should not include click-and-drag events that pass over widgets. For click-and-drag, it is the act of releasing the mouse button that "ends" the selection.

> 💬 The web uses `selectstart` and `selectionchange` events. However, it might be nice to respond to a selection being ended so I added `SelectionEnd`. My question is, do we include a `SelectionChanged` event that is invoked on all widgets when the selection changes? And are there other events we might want, such as `OnSelect` and `OnDeselect`? 

#### 4.3. Indicating the Selection

Obviously all of this is no good if we can't actually display the selection to the user. There's two types of displays we'll need to cover: collapsed ranges and non-collapsed ranges.

Both only apply to text-based widgets. We can ignore all non-text widgets when handling these selection indicators.

Furthermore, we'll likely want to handle them as *pseudo-elements*. In other words, they shouldn't be physical widgets existing in the widget tree and instead primitives sent directly to the renderer. There are two reasons for this:

1. It makes the whole system "automatic" (users who make their own custom `Text` widget don't need to integrate all our custom logic to achieve the same effect)
2. It also might be more performant since it reduces the number of tree mutations we'd need to make (due to these pseudo-elements not being actual nodes in the tree)

> 💬 Is this a good way of handling it? Is there a better way?

##### 4.3.1. Collapsed Ranges

Collapsed ranges display what's known as the text insertion cursor, or *caret*. This is usually only displayed in editable content. Unfortunately we don't have a distinguishing or handling editable content directly (it has to be managed by a widget like `TextBox`). Such a feature could be added in the future using a text diff algorithm, but we'll consider that out of scope for now— though, it should be not too difficult to add onto the systems we develop here.

Until then, any text widget with an appropriate [`caret`](#442-caret) style will display the caret, assuming the selection range is located within it.

> 💬 Since the caret is a pseudo-element, we might be able to even incorporate blinking directly in the shader. Although, I'm not sure how feasible that is or how we would handle pausing the blink when the caret moves (as is standard behavior).

##### 4.3.2. Non-Collapsed Ranges

Non-collapsed ranges will need to display the full range of the selection. This is normally done by adding a semi-transparent background to the selected text. We cannot just set the `background_color` style of the widget, though, since not all of it might be selected.

To get around this, we will need to create one quad for each line that spans the selected region. Another option would be to render at most three quads: one for the start selection, one for the end, and one for all the content in the middle. However, this assumes that the line height is set in such a way that warrants a seamless selection area. It also runs into an issue with self-directed widgets that might not align with other content. Therefore, I think the multi-quad approach will be the simplest and most versatile.

#### 4.4. Selection Styles

##### 4.4.1. `select`

This style property controls how content is selected. This can be applied to any widget.

###### Values

| Variant           | Description                                                  |
| ----------------- | ------------------------------------------------------------ |
| `Select::Normal`  | Allows for normal selection text selection. This is the default value. |
| `Select::Contain` | Restricts a selection started within this widget or its descendants to the widget |
| `Select::All`     | Selects/deselects all contents of this widget at once        |
| `Select::None`    | This widget and all its descendants are not selectable.      |

> All values are values can be overridden. For example, setting a parent to `Select::None` and its child to `Select::Normal` will still allow the child to be selected.

> The specs for these values are based on the [web equivalent values](https://developer.mozilla.org/en-US/docs/Web/CSS/user-select#syntax). Those specs also suggest that this property should not be inheritable and that its default be `auto`. The reasoning for this is to enforce consistency with editable content and pseudo-elements. Since we don't really have those, we can just ignore this and allow inheritance.

##### 4.4.2. `caret`

This style property controls how the text insertion cursor (caret) is displayed. This can be applied to any `RenderCommand::Text` widget.

> While this is generally only useful to editable content, we don't have a great way of making that kind of distinction like HTML [can](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/contenteditable). So we'll just allow it for any text widget that opts in.

###### Values

| Variant               | Description                                                  |
| --------------------- | ------------------------------------------------------------ |
| `Caret::None`         | No caret is displayed. This is the default value.            |
| `Caret::Bar`          | A vertical bar is displayed after the character[^1] at the collapsed position |
| `Caret::Under`        | A horizontal line is displayed below the character[^1] at the collapsed position |
| `Caret::Block`        | A background is displayed behind the character[^1] at the collapsed position |
| `Caret::Custom(char)` | A custom character is displayed after the character[^1] at the collapsed position |

##### 4.4.3. `caret_color`

This style property controls the color of the caret (when displayed). This can be applied to any `RenderCommand::Text` widget.

Accepts any `Color` value.

##### 4.4.4. `selection_background_color`

This style property controls the background color of the selection. This does not change the background color of *all* text in the widget, just the selected range. This can be applied to any `RenderCommand::Text` widget.

Accepts any `Color` value.

##### 4.4.5. `selection_color`

This style property controls the text color of the selection. This does not change the text color of *all* text in the widget, just the selected range. This can be applied to any `RenderCommand::Text` widget.

Accepts any `Color` value.

> By default this should take on the current `color` value.

## Implementation Guide

Below is a guide for how we could implement this RFC. The exact details of the implementation can be decided in the PRs themselves or in separate RFC documents. This list should also be taken with a grain of salt as it might make sense to do things differently as the implementation process begins.

### Small Changes

These are changes that are quick to implement or don't interact with other systems/APIs too much.

1. Add the appropriate methods to `Node` ([reference](#13-node-methods))
2. Create `Range` and `RangeBound` structs ([reference](#2-defining-the-range))

### Moderate Changes

These are changes that might be slightly more difficult to implement or have limited interaction with other systems/APIs.

1. Use grapheme clusters for font sizing ([reference](#character))
2. Implement text layout caching and sizing ([reference](#32-text-nodes))

### Large Changes

These are changes that are large in scope, difficult to implement, or touch a large number of other systems/APIs.

1. Add selection object ([reference](#41-the-selection-api))
2. Add selection indicators ([reference](#43-indicating-the-selection))
   1. Add selection styles ([reference](#44-selection-styles))
3. Add selection events ([reference](#424-selection-events))
   1. Handle user interaction ([reference](#42-creating-the-selection))

---

[^1]: By character, remember we are referring to the actual grapheme cluster, not any individual byte.
[^2]: A word in this sense is a sequence of characters[^1] that are non-breaking, meaning they can't be split across lines.
[^3]: Assume `usize` is 8 bytes
[^4]: Estimated, based on known variables and not including additional overhead (such as from `Vec` itself)