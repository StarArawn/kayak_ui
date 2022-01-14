use super::traits::*;
use super::releasable::*;
use super::binding_context::*;

use futures::prelude::*;
use ::desync::*;

use std::sync::*;

#[allow(dead_code)]
///
/// Uses a stream to update a binding
/// 
pub fn bind_stream<S, Value, UpdateFn>(stream: S, initial_value: Value, update: UpdateFn) -> StreamBinding<Value>
where   S:          'static+Send+Stream+Unpin,
        Value:      'static+Send+Clone+PartialEq,
        UpdateFn:   'static+Send+FnMut(Value, S::Item) -> Value,
        S::Item:    Send {
    // Create the content of the binding
    let value       = Arc::new(Mutex::new(initial_value));
    let core        = StreamBindingCore {
        value:          Arc::clone(&value),
        notifications:  vec![]
    };

    let core        = Arc::new(Desync::new(core));
    let mut update  = update;

    // Send in the stream
    pipe_in(Arc::clone(&core), stream, 
        move |core, next_item| {
            // Only lock the value while updating it
            let need_to_notify = {
                // Update the value
                let mut value = core.value.lock().unwrap();
                let new_value = update((*value).clone(), next_item);

                if new_value != *value {
                    // Update the value in the core
                    *value = new_value;

                    // Notify anything that's listening
                    true
                } else {
                    false
                }
            };

            // If the update changed the value, then call the notifications (with the lock released, in case any try to read the value)
            if need_to_notify {
                core.notifications.retain(|notify| notify.is_in_use());
                core.notifications.iter().for_each(|notify| { notify.mark_as_changed(); });
            }

            Box::pin(future::ready(()))
        });
    
    StreamBinding {
        core:   core,
        value:  value
    }
}

///
/// Binding that represents the result of binding a stream to a value
/// 
#[derive(Clone)]
pub struct StreamBinding<Value: Send> {
    /// The core of the binding (where updates are streamed and notifications sent)
    core: Arc<Desync<StreamBindingCore<Value>>>,

    /// The current value of the binding
    value: Arc<Mutex<Value>>
}

///
/// The data stored with a stream binding
/// 
struct StreamBindingCore<Value: Send> {
    /// The current value of this binidng
    value: Arc<Mutex<Value>>,

    /// The items that should be notified when this binding changes
    notifications: Vec<ReleasableNotifiable>
}

impl<Value: 'static+Send+Clone> Bound<Value> for StreamBinding<Value> {
    ///
    /// Retrieves the value stored by this binding
    ///
    fn get(&self) -> Value {
        BindingContext::add_dependency(self.clone());

        let value = self.value.lock().unwrap();
        (*value).clone()
    }
}

impl<Value: 'static+Send> Changeable for StreamBinding<Value> {
    ///
    /// Supplies a function to be notified when this item is changed
    ///
    fn when_changed(&self, what: Arc<dyn Notifiable>) -> Box<dyn Releasable> {
        // Create the notification object
        let releasable = ReleasableNotifiable::new(what);
        let notifiable = releasable.clone_as_owned();

        // Send to the core
        self.core.desync(move |core| {
            core.notifications.push(notifiable);
        });

        // Return the releasable object
        Box::new(releasable)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::notify_fn::*;

    use futures::stream;
    use futures::executor;
    use futures::channel::mpsc;

    use std::thread;
    use std::time::Duration;

    #[test]
    pub fn stream_in_all_values() {
        // Stream with the values '1,2,3'
        let stream  = vec![1, 2, 3];
        let stream  = stream::iter(stream.into_iter());

        // Send the stream to a new binding
        let binding = bind_stream(stream, 0, |_old_value, new_value| new_value);

        thread::sleep(Duration::from_millis(10));

        // Binding should have the value of the last value in the stream
        assert!(binding.get() == 3);
    }

    #[test]
    pub fn stream_processes_updates() {
        // Stream with the values '1,2,3'
        let stream  = vec![1, 2, 3];
        let stream  = stream::iter(stream.into_iter());

        // Send the stream to a new binding (with some processing)
        let binding = bind_stream(stream, 0, |_old_value, new_value| new_value + 42);

        thread::sleep(Duration::from_millis(10));

        // Binding should have the value of the last value in the stream
        assert!(binding.get() == 45);
    }

    #[test]
    pub fn notifies_on_change() {
        // Create somewhere to send our notifications
        let (mut sender, receiver) = mpsc::channel(0);

        // Send the receiver stream to a new binding
        let binding         = bind_stream(receiver, 0, |_old_value, new_value| new_value);

        // Create the notification
        let notified        = Arc::new(Mutex::new(false));
        let also_notified   = Arc::clone(&notified);

        binding.when_changed(notify(move || *also_notified.lock().unwrap() = true)).keep_alive();

        // Should be initially un-notified
        thread::sleep(Duration::from_millis(5));
        assert!(*notified.lock().unwrap() == false);

        executor::block_on(async {
            // Send a value to the sender
            sender.send(42).await.unwrap();

            // Should get notified
            thread::sleep(Duration::from_millis(5));
            assert!(*notified.lock().unwrap() == true);
            assert!(binding.get() == 42);
        })
    }

    #[test]
    pub fn no_notification_on_no_change() {
        // Create somewhere to send our notifications
        let (mut sender, receiver) = mpsc::channel(0);

        // Send the receiver stream to a new binding
        let binding = bind_stream(receiver, 0, |_old_value, new_value| new_value);

        // Create the notification
        let notified        = Arc::new(Mutex::new(false));
        let also_notified   = Arc::clone(&notified);

        binding.when_changed(notify(move || *also_notified.lock().unwrap() = true)).keep_alive();

        // Should be initially un-notified
        thread::sleep(Duration::from_millis(5));
        assert!(*notified.lock().unwrap() == false);

        executor::block_on(async {
            // Send a value to the sender. This leaves the final value the same, so no notification should be generated.
            sender.send(0).await.unwrap();

            // Should not get notified
            thread::sleep(Duration::from_millis(5));
            assert!(*notified.lock().unwrap() == false);
            assert!(binding.get() == 0);
        });
    }
}
