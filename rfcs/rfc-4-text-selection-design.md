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

### 1. Retrieving Text

Before we start discussing how we select text and all that, we need to discuss how we even process our widgets textually. Thankfully we already have a system in place to determine if a widget displays text and what text content it displays. All we need to do is check if a widget has a `RenderCommand::Text` render command.

### 2. Defining the Range

#### 2.1. Bounds

Since a [range](#range) is just a start and an end point, we first need a way of identifying those points. The big questions we need to ask are:

* *What node does this position belong to?*
* *What character[^1] is this position pointing to?*

As long as we have the answer to those two questions we'll know exactly where the position is, and consequently, how our range is defined. Luckily, both are really simple to represent: all we need is a widget's `id` and the character[^1] index from the start of the widget's text (i.e. the number of captured characters).

> This can also be used for non-text nodes as well. In that case, the offset would just refer to the number of captured *children* rather than characters.

This can succinctly be stored in a struct like:

```rust
struct RangeBound {
  node: Index,
  offset: usize
}
```

> In examples throughout this RFC, we'll use the short-form notation of `(Node, Offset)`. So a bound matching the node of ID 2 at offset 3 would be notated as (2, 3).

Given any two `RangeBound` objects, we can define an actual range like so:

```rust
struct Range {
  start: RangeBound,
  end: RangeBound
}
```

If we allow ourselves to jump ahead for a moment, we might wonder if text selection will always have a start and end. Could one exist without the other?

It might be possible that we only know the start or end of a range and not both. However, recall that a range is perfectly valid even if its start and end are the same point. Therefore, if we can only define the range with a single point, we can simply set both the start *and* end to that point.

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

> In fact, we've been doing this already. A node that's fully captured by a range can be seen as sub-range from (Node, 0) to (Node, Length-1). And we can apply the same rules to the children as well, which is why we capture *all* of Node 3 in the example above. This is pretty much what happens when a range spans across a full node, such as in [Example 2.2.1](#example-2.2.1).

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

There are two cases to consider: non-text nodes, text nodes, and mixed nodes.

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





---

[^1]: By character, remember we are referring to the actual grapheme cluster, not any individual byte.
[^2]: A word in this sense is a sequence of characters[^1] that are non-breaking, meaning they can't be split across lines.
[^3]: Assume `usize` is 8 bytes
[^4]: Estimated, based on known variables and not including additional overhead (such as from `Vec` itself)