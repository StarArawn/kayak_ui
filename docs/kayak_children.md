## Intro
1. We have a simple tree that stores entities.
2. Nodes in the tree need to have consistent identifiers(I.E. entity id's).

## Problems
1. Children passing - Children are entities that are passed from higher up in the tree to some place lower in the tree. Normally children are directly attached to their parent, but because of our tree structure we often need to spawn entities ahead of time because the parent has not yet spawned. Example xml tree:

Root:
```xml
<a>
    <b>
        <c />
    </b>
</a>
```
B:
```xml
<d>
    {children}
</d>
```

Here the widget B is wrapped around C but the actual children are a child of D. 

How can we handle children?

### 1. Closures - A function that can be passed down the tree. 
Pros:
- Essentially differed adding and spawning of widgets until later.
- No need to worry about if the entity ID is correct in the tree just pull it out.

Cons:
- Closures require ownership to move data into them. This might seem small but remember we have a tree again so quite quickly it becomes challenging:
```rust
// Imagine some data that has clone only.
let some_data = SomeData { foo };
parent.children = Children::new(move ||
    let some_data = some_data.clone();
    let child_1 = create_widget();
    child_1.children = Children::new(move || {
        // We might need to clone again if child_2 has children that need this data?
        // let some_data = some_data.clone();
        // let child_2 = create_widget(some_data.foo); // OOPS error.
        let child_2 = create_widget(some_data.foo);
    })
);
```
This is a pretty massive con in my opinion and one we should try to avoid.

## Alternatives

### 2. Ordered tree
Another solution is to keep track of where the entity id's are spawned and make them consistent at their spawn point. Then we don't need to pass data down the tree only entity id's.

Process:
1. For each widget spawned add it to a new tree with it's parent set to the widget that spawned it.
2. Make sure this ordered tree stays in sync with additions of entities.
3. Pass children down as just id's that get added to the tree in their actual parent widget.

So for our same example above with(a, b, c, d). Our tree looks like:

```
├── A 1
|   └── B 2
|       └── D 4    
|           └── C 3
```

Our ordered tree looks like:
```
├── A 1
|   ├── B 2
|       └── D 4   
|   └── C 3      
```

Now with a lack of closures we only need to clone, copy, or move data when we specifically need to. When we re-render the tree a second time we look at our ordered tree first to see if we have a matching entity in the slot we are trying to render to.

This sounds easy in practice but there are also some big cons:

Cons:
- Complexity increased.
- Can cause issues if your tree and ordered tree get out of sync.
