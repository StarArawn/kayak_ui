use super::notify_fn::*;
use super::traits::*;

use futures::task;
use futures::task::Poll;
use futures::*;

use ::desync::*;

use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::*;

///
/// The state of the binding for a follow stream
///
#[derive(Copy, Clone)]
#[allow(dead_code)]
enum FollowState {
    Unchanged,
    Changed,
}

///
/// Core data structures for a follow stream
///
struct FollowCore<TValue, Binding: Bound<TValue>> {
    /// Changed if the binding value has changed, or Unchanged if it is not changed
    state: FollowState,

    /// What to notify when this item is changed
    notify: Option<task::Waker>,

    /// The binding that this is following
    binding: Arc<Binding>,

    /// Value is stored in the binding
    value: PhantomData<TValue>,
}

///
/// Stream that follows the values of a binding
///
pub struct FollowStream<TValue: Send + Unpin, Binding: Bound<TValue>> {
    /// The core of this future
    core: Arc<Desync<FollowCore<TValue, Binding>>>,

    /// Lifetime of the watcher
    _watcher: Box<dyn Releasable>,
}

impl<TValue: 'static + Send + Unpin, Binding: 'static + Bound<TValue>> Stream
    for FollowStream<TValue, Binding>
{
    type Item = TValue;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<Option<Self::Item>> {
        // If the core is in a 'changed' state, return the binding so we can fetch it
        // Want to fetch the binding value outside of the lock as it can potentially change during calculation
        let binding = self.core.sync(|core| {
            match core.state {
                FollowState::Unchanged => {
                    // Wake this future when changed
                    core.notify = Some(cx.waker().clone());
                    None
                }

                FollowState::Changed => {
                    // Value has changed since we were last notified: return the changed value
                    core.state = FollowState::Unchanged;
                    Some(Arc::clone(&core.binding))
                }
            }
        });

        if let Some(binding) = binding {
            Poll::Ready(Some(binding.get()))
        } else {
            Poll::Pending
        }
    }
}

///
/// Creates a stream from a binding
///
#[allow(dead_code)]
pub fn follow<TValue: 'static + Send + Unpin, Binding: 'static + Bound<TValue>>(
    binding: Binding,
) -> FollowStream<TValue, Binding> {
    // Generate the initial core
    let core = FollowCore {
        state: FollowState::Changed,
        notify: None,
        binding: Arc::new(binding),
        value: PhantomData,
    };

    // Notify whenever the binding changes
    let core = Arc::new(Desync::new(core));
    let weak_core = Arc::downgrade(&core);
    let watcher = core.sync(move |core| {
        core.binding.when_changed(notify(move || {
            if let Some(core) = weak_core.upgrade() {
                let task = core.sync(|core| {
                    core.state = FollowState::Changed;
                    core.notify.take()
                });
                task.map(|task| task.wake());
            }
        }))
    });

    // Create the stream
    FollowStream {
        core: core,
        _watcher: watcher,
    }
}

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;

    use futures::executor;
    use futures::task::{waker_ref, ArcWake, Context};

    use std::thread;
    use std::time::Duration;

    struct NotifyNothing;
    impl ArcWake for NotifyNothing {
        fn wake_by_ref(_arc_self: &Arc<Self>) {
            // zzz
        }
    }

    #[test]
    fn follow_stream_has_initial_value() {
        let binding = bind(1);
        let bind_ref = BindRef::from(binding.clone());
        let mut stream = follow(bind_ref);

        executor::block_on(async {
            assert!(stream.next().await == Some(1));
        });
    }

    #[test]
    fn follow_stream_updates() {
        let binding = bind(1);
        let bind_ref = BindRef::from(binding.clone());
        let mut stream = follow(bind_ref);

        executor::block_on(async {
            assert!(stream.next().await == Some(1));
            binding.set(2);
            assert!(stream.next().await == Some(2));
        });
    }

    #[test]
    fn computed_updates_during_read() {
        // Computed value that takes a while to calculate (so we can always 'lose' the race between reading the value and starting a new update)
        let binding = bind(1);
        let bind_ref = BindRef::from(binding.clone());
        let computed = computed(move || {
            let val = bind_ref.get();
            thread::sleep(Duration::from_millis(300));
            val
        });
        let mut stream = follow(computed);

        // Read from the stream in the background
        let reader = Desync::new(vec![]);
        let read_values = reader.after(
            async move {
                let result = vec![stream.next().await, stream.next().await];
                result
            },
            |val, read_val| {
                *val = read_val;
            },
        );

        // Short delay so the reader starts
        thread::sleep(Duration::from_millis(10));

        // Update the binding
        binding.set(2);

        // Wait for the values to be read from the stream
        let values_read_from_stream = reader.sync(|val| val.clone());

        // First read should return '1'
        assert!(values_read_from_stream[0] == Some(1));

        // Second read should return '2'
        assert!(values_read_from_stream[1] == Some(2));

        // Finish the read_values future
        executor::block_on(read_values).unwrap();
    }

    #[test]
    fn stream_is_unready_after_first_read() {
        let binding = bind(1);
        let bind_ref = BindRef::from(binding.clone());
        let waker = Arc::new(NotifyNothing);
        let waker = waker_ref(&waker);
        let mut context = Context::from_waker(&waker);
        let mut stream = follow(bind_ref);

        assert!(stream.poll_next_unpin(&mut context) == Poll::Ready(Some(1)));
        assert!(stream.poll_next_unpin(&mut context) == Poll::Pending);
    }

    #[test]
    fn stream_is_immediately_ready_after_write() {
        let binding = bind(1);
        let bind_ref = BindRef::from(binding.clone());
        let waker = Arc::new(NotifyNothing);
        let waker = waker_ref(&waker);
        let mut context = Context::from_waker(&waker);
        let mut stream = follow(bind_ref);

        assert!(stream.poll_next_unpin(&mut context) == Poll::Ready(Some(1)));
        binding.set(2);
        assert!(stream.poll_next_unpin(&mut context) == Poll::Ready(Some(2)));
    }

    #[test]
    fn will_wake_when_binding_is_updated() {
        let binding = bind(1);
        let bind_ref = BindRef::from(binding.clone());
        let mut stream = follow(bind_ref);

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            binding.set(2);
        });

        executor::block_on(async {
            assert!(stream.next().await == Some(1));
            assert!(stream.next().await == Some(2));
        })
    }
}
