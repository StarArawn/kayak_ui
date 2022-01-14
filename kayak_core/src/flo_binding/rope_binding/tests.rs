use crate::flo_binding::*;

use flo_rope::*;

use futures::executor;
use futures::prelude::*;

#[test]
fn mutable_rope_sends_changes_to_stream() {
    // Create a rope that copies changes from a mutable rope
    let mut mutable_rope = RopeBindingMut::<usize, ()>::new();
    let mut rope_stream = mutable_rope.follow_changes();

    // Write some data to the mutable rope
    mutable_rope.replace(0..0, vec![1, 2, 3, 4]);

    // Should get sent to the stream
    executor::block_on(async move {
        let next = rope_stream.next().await;

        assert!(next == Some(RopeAction::Replace(0..0, vec![1, 2, 3, 4])));
    });
}

#[test]
fn pull_from_mutable_binding() {
    // Create a rope that copies changes from a mutable rope
    let mut mutable_rope = RopeBindingMut::<usize, ()>::new();
    let rope_copy = RopeBinding::from_stream(mutable_rope.follow_changes());
    let mut rope_stream = rope_copy.follow_changes();

    // Write some data to the mutable rope
    mutable_rope.replace(0..0, vec![1, 2, 3, 4]);

    // Wait for the change to arrive at the copy
    executor::block_on(async move {
        let next = rope_stream.next().await;
        assert!(next == Some(RopeAction::Replace(0..0, vec![1, 2, 3, 4])))
    });

    // Read from the copy
    assert!(rope_copy.len() == 4);
    assert!(rope_copy.read_cells(0..4).collect::<Vec<_>>() == vec![1, 2, 3, 4]);
}
