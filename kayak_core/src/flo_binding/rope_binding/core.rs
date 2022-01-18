use crate::flo_binding::releasable::*;
use crate::flo_binding::rope_binding::stream_state::*;

use flo_rope::*;
use futures::task::*;

///
/// The core of a rope binding represents the data that's shared amongst all ropes
///
pub(super) struct RopeBindingCore<Cell, Attribute>
where
    Cell: Clone + PartialEq,
    Attribute: Clone + PartialEq + Default,
{
    /// The number of items that are using hte core
    pub(super) usage_count: usize,

    /// The rope that stores this binding
    pub(super) rope: PullRope<AttributedRope<Cell, Attribute>, Box<dyn Fn() -> () + Send + Sync>>,

    /// The states of any streams reading from this rope
    pub(super) stream_states: Vec<RopeStreamState<Cell, Attribute>>,

    #[allow(dead_code)]
    /// The next ID to assign to a stream state
    pub(super) next_stream_id: usize,

    // List of things to call when this binding changes
    pub(super) when_changed: Vec<ReleasableNotifiable>,
}

impl<Cell, Attribute> RopeBindingCore<Cell, Attribute>
where
    Cell: 'static + Send + Unpin + Clone + PartialEq,
    Attribute: 'static + Send + Sync + Clone + Unpin + PartialEq + Default,
{
    ///
    /// If there are any notifiables in this object that aren't in use, remove them
    ///
    pub(super) fn filter_unused_notifications(&mut self) {
        self.when_changed
            .retain(|releasable| releasable.is_in_use());
    }

    ///
    /// Callback: the rope has changes to pull
    ///
    pub(super) fn on_pull(&mut self) {
        // Clear out any notifications that are not being used any more
        self.filter_unused_notifications();

        // Notify anything that's listening
        for notifiable in &self.when_changed {
            notifiable.mark_as_changed();
        }

        // Wake any streams that are waiting for changes to be pulled
        for stream in self.stream_states.iter_mut() {
            let waker = stream.waker.take();

            if let Some(waker) = waker {
                // Wake the stream so that it pulls the changes
                waker.wake();
            } else {
                // If the stream is trying to sleep, make sure it wakes up immediately
                stream.needs_pull = true;
            }
        }
    }

    ///
    /// Pulls values from the rope and send to all attached streams
    ///
    pub(super) fn pull_rope(&mut self) {
        // Stop the streams from waking up (no changes pending)
        for stream in self.stream_states.iter_mut() {
            stream.needs_pull = false;
        }

        // Collect the actions
        let actions = self.rope.pull_changes().collect::<Vec<_>>();

        // Don't wake anything if there are no actions to perform
        if actions.len() == 0 {
            return;
        }

        // Push to each stream
        for stream in self.stream_states.iter_mut() {
            stream.pending_changes.extend(actions.iter().cloned());
        }

        // Wake all of the streams
        for stream in self.stream_states.iter_mut() {
            let waker = stream.waker.take();
            waker.map(|waker| waker.wake());
        }
    }

    ///
    /// Wakes a particular stream when the rope changes
    ///
    pub(super) fn wake_stream(&mut self, stream_id: usize, waker: Waker) {
        self.stream_states
            .iter_mut()
            .filter(|state| state.identifier == stream_id)
            .nth(0)
            .map(move |state| {
                if !state.needs_pull {
                    // There are no pending values so we should wait for the rope to pull some extra data

                    // Wake the stream when there's some more data to receive
                    state.waker = Some(waker);
                } else {
                    // There are pending values so we should immediately re-awaken the stream

                    // Disable the waker in case there's a stale one
                    state.waker = None;

                    // Wake the stream so it reads the next value
                    waker.wake();
                }
            });
    }
}
