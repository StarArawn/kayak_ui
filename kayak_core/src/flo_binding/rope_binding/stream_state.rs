use flo_rope::*;
use futures::task::*;

use std::collections::VecDeque;

///
/// The state of a stream that is reading from a rope binding core
///
pub(super) struct RopeStreamState<Cell, Attribute>
where
    Cell: Clone + PartialEq,
    Attribute: Clone + PartialEq + Default,
{
    /// The identifier for this stream
    pub(super) identifier: usize,

    /// The waker for the current stream
    pub(super) waker: Option<Waker>,

    /// The changes that are waiting to be sent to this stream
    pub(super) pending_changes: VecDeque<RopeAction<Cell, Attribute>>,

    /// True if the rope has indicated there are changes waiting to be pulled
    pub(super) needs_pull: bool,
}
