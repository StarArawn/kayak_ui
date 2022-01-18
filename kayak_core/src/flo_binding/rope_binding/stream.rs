use crate::flo_binding::rope_binding::core::*;

use ::desync::*;
use flo_rope::*;
use futures::future::BoxFuture;
use futures::prelude::*;
use futures::task::*;

use std::collections::VecDeque;
use std::mem;
use std::pin::*;
use std::sync::*;

///
/// A rope stream monitors a rope binding, and supplies them as a stream so they can be mirrored elsewhere
///
/// An example of a use for a rope stream is to send updates from a rope to a user interface.
///
pub struct RopeStream<Cell, Attribute>
where
    Cell: 'static + Send + Unpin + Clone + PartialEq,
    Attribute: 'static + Send + Sync + Clone + Unpin + PartialEq + Default,
{
    /// The identifier for this stream
    pub(super) identifier: usize,

    /// The core of the rope
    pub(super) core: Arc<Desync<RopeBindingCore<Cell, Attribute>>>,

    /// A future that will return the next poll result
    pub(super) poll_future:
        Option<BoxFuture<'static, Poll<Option<VecDeque<RopeAction<Cell, Attribute>>>>>>,

    /// The actions that are currently being drained through this stream
    pub(super) draining: VecDeque<RopeAction<Cell, Attribute>>,
}

impl<Cell, Attribute> Stream for RopeStream<Cell, Attribute>
where
    Cell: 'static + Send + Unpin + Clone + PartialEq,
    Attribute: 'static + Send + Sync + Clone + Unpin + PartialEq + Default,
{
    type Item = RopeAction<Cell, Attribute>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        ctxt: &mut Context<'_>,
    ) -> Poll<Option<RopeAction<Cell, Attribute>>> {
        // If we've got a set of actions we're already reading, then return those as fast as we can
        if self.draining.len() > 0 {
            return Poll::Ready(self.draining.pop_back());
        }

        // If we're waiting for the core to return to us, borrow the future from there
        let poll_future = self.poll_future.take();
        let mut poll_future = if let Some(poll_future) = poll_future {
            // We're already waiting for the core to get back to us
            poll_future
        } else {
            // Ask the core for the next stream state
            let stream_id = self.identifier;

            self.core
                .future_desync(move |core| {
                    async move {
                        // Pull any pending changes from the rope
                        core.pull_rope();

                        // Find the state of this stream
                        let stream_state = core
                            .stream_states
                            .iter_mut()
                            .filter(|state| state.identifier == stream_id)
                            .nth(0)
                            .unwrap();

                        // Check for data
                        if stream_state.pending_changes.len() > 0 {
                            // Return the changes to the waiting stream
                            let mut changes = VecDeque::new();
                            mem::swap(&mut changes, &mut stream_state.pending_changes);

                            Poll::Ready(Some(changes))
                        } else if core.usage_count == 0 {
                            // No changes, and nothing is using the core any more
                            Poll::Ready(None)
                        } else {
                            // No changes are waiting
                            Poll::Pending
                        }
                    }
                    .boxed()
                })
                .map(|result| {
                    // Error would indicate the core had gone away before the request should complete, so we signal this as an end-of-stream event
                    match result {
                        Ok(result) => result,
                        Err(_) => Poll::Ready(None),
                    }
                })
                .boxed()
        };

        // Ask the future for the latest update on this stream
        let future_result = poll_future.poll_unpin(ctxt);

        match future_result {
            Poll::Ready(Poll::Ready(Some(actions))) => {
                if actions.len() == 0 {
                    // Nothing waiting: need to wait until the rope signals a 'pull' event
                    let waker = ctxt.waker().clone();
                    let stream_id = self.identifier;

                    self.core.desync(move |core| {
                        core.wake_stream(stream_id, waker);
                    });

                    Poll::Pending
                } else {
                    // Have some actions ready
                    self.draining = actions;
                    Poll::Ready(self.draining.pop_back())
                }
            }

            Poll::Ready(Poll::Ready(None)) => Poll::Ready(None),
            Poll::Ready(Poll::Pending) => {
                // Wake when the rope generates a 'pull' event
                let waker = ctxt.waker().clone();
                let stream_id = self.identifier;

                self.core.desync(move |core| {
                    core.wake_stream(stream_id, waker);
                });

                Poll::Pending
            }

            Poll::Pending => {
                // Poll the future again when it notifies
                self.poll_future = Some(poll_future);
                Poll::Pending
            }
        }
    }
}

impl<Cell, Attribute> Drop for RopeStream<Cell, Attribute>
where
    Cell: 'static + Send + Unpin + Clone + PartialEq,
    Attribute: 'static + Send + Sync + Clone + Unpin + PartialEq + Default,
{
    fn drop(&mut self) {
        // Remove the stream state when the stream is no more
        let dropped_stream_id = self.identifier;
        self.core.desync(move |core| {
            core.stream_states
                .retain(|state| state.identifier != dropped_stream_id);
        });
    }
}
