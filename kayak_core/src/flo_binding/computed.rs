use super::binding_context::*;
use super::releasable::*;
use super::traits::*;

use std::mem;
use std::sync::*;

///
/// Represents a computed value
///
#[derive(Clone)]
enum ComputedValue<Value: 'static + Clone> {
    Unknown,
    Cached(Value),
}

use self::ComputedValue::*;

///
/// Core representation ofa computed binding
///
struct ComputedBindingCore<Value: 'static + Clone, TFn>
where
    TFn: 'static + Fn() -> Value,
{
    /// Function to call to recalculate this item
    calculate_value: TFn,

    /// Most recent cached value
    latest_value: ComputedValue<Value>,

    /// If there's a notification attached to this item, this can be used to release it
    existing_notification: Option<Box<dyn Releasable>>,

    /// What to call when the value changes
    when_changed: Vec<ReleasableNotifiable>,
}

impl<Value: 'static + Clone, TFn> ComputedBindingCore<Value, TFn>
where
    TFn: 'static + Fn() -> Value,
{
    ///
    /// Creates a new computed binding core item
    ///
    pub fn new(calculate_value: TFn) -> ComputedBindingCore<Value, TFn> {
        ComputedBindingCore {
            calculate_value: calculate_value,
            latest_value: Unknown,
            existing_notification: None,
            when_changed: vec![],
        }
    }

    ///
    /// Marks the value as changed, returning true if the value was removed
    ///
    pub fn mark_changed(&mut self) -> bool {
        match self.latest_value {
            Unknown => false,
            _ => {
                self.latest_value = Unknown;
                true
            }
        }
    }

    ///
    /// Retrieves a copy of the list of notifiable items for this value
    ///
    pub fn get_notifiable_items(&self) -> Vec<ReleasableNotifiable> {
        self.when_changed
            .iter()
            .map(|item| item.clone_for_inspection())
            .collect()
    }

    ///
    /// If there are any notifiables in this object that aren't in use, remove them
    ///
    pub fn filter_unused_notifications(&mut self) {
        self.when_changed
            .retain(|releasable| releasable.is_in_use());
    }

    ///
    /// Returns the current value (or 'Unknown' if it needs recalculating)
    ///
    pub fn get(&self) -> ComputedValue<Value> {
        self.latest_value.clone()
    }

    ///
    /// Recalculates the latest value
    ///
    pub fn recalculate(&mut self) -> (Value, BindingDependencies) {
        // Perform the binding in a context to get the value and the dependencies
        let (result, dependencies) = BindingContext::bind(|| (self.calculate_value)());

        // Update the latest value
        self.latest_value = Cached(result.clone());

        // Pass on the result
        (result, dependencies)
    }
}

impl<Value: 'static + Clone, TFn> Drop for ComputedBindingCore<Value, TFn>
where
    TFn: 'static + Fn() -> Value,
{
    fn drop(&mut self) {
        // No point receiving any notifications once the core has gone
        // (The notification can still fire if it has a weak reference)
        if let Some(ref mut existing_notification) = self.existing_notification {
            existing_notification.done()
        }
    }
}

///
/// Represents a binding to a value that is computed by a function
///
pub struct ComputedBinding<Value: 'static + Clone, TFn>
where
    TFn: 'static + Fn() -> Value,
{
    /// The core where the binding data is stored
    core: Arc<Mutex<ComputedBindingCore<Value, TFn>>>,
}

impl<Value: 'static + Clone + Send, TFn> ComputedBinding<Value, TFn>
where
    TFn: 'static + Send + Sync + Fn() -> Value,
{
    ///
    /// Creates a new computable binding
    ///
    pub fn new(calculate_value: TFn) -> ComputedBinding<Value, TFn> {
        // Computed bindings created in a binding context will likely not be
        // retained, so things won't update as you expect.
        //
        // We could add some special logic to retain them here, or we could
        // just panic. Panicking is probably better as really what should be
        // done is to evaluate the content of the computed value directly.
        // This can happen if we call a function that returns a binding and
        // it creates one rather than returning an existing one.
        BindingContext::panic_if_in_binding_context("Cannot create computed bindings in a computed value calculation function (you should evaluate the value directly rather than create bindings)");

        // Create the binding
        ComputedBinding {
            core: Arc::new(Mutex::new(ComputedBindingCore::new(calculate_value))),
        }
    }

    ///
    /// Creates a new computable binding within another binding
    ///
    /// Normally this is considered an error (if the binding is not held anywhere
    /// outside of the context, it will never generate an update). `new` panics
    /// if it's called from within a context for this reason.
    ///
    /// If the purpose of a computed binding is to return other bindings, this
    /// limitation does not apply, so this call is available
    ///
    pub fn new_in_context(calculate_value: TFn) -> ComputedBinding<Value, TFn> {
        // Create the binding
        ComputedBinding {
            core: Arc::new(Mutex::new(ComputedBindingCore::new(calculate_value))),
        }
    }

    ///
    /// Marks this computed binding as having changed
    ///
    fn mark_changed(&self, force_notify: bool) {
        // We do the notifications and releasing while the lock is not retained
        let (notifiable, releasable) = {
            // Get the core
            let mut core = self.core.lock().unwrap();

            // Mark it as changed
            let actually_changed = core.mark_changed() || force_notify;

            core.filter_unused_notifications();

            // Get the items that need changing (once we've notified our dependencies that we're changed, we don't need to notify them again until we get recalculated)
            let notifiable = if actually_changed {
                core.get_notifiable_items()
            } else {
                vec![]
            };

            // Extract the releasable so we can release it after the lock has gone
            let mut releasable: Option<Box<dyn Releasable>> = None;
            mem::swap(&mut releasable, &mut core.existing_notification);

            // These values are needed outside of the lock
            (notifiable, releasable)
        };

        // Don't want any more notifications from this source
        releasable.map(|mut releasable| releasable.done());

        // Notify anything that needs to be notified that this has changed
        for to_notify in notifiable {
            to_notify.mark_as_changed();
        }
    }

    ///
    /// Mark this item as changed whenever 'to_monitor' is changed
    /// Core should already be locked, returns true if the value is already changed and we should immediately notify
    ///
    fn monitor_changes(
        &self,
        core: &mut ComputedBindingCore<Value, TFn>,
        to_monitor: &mut BindingDependencies,
    ) -> bool {
        // We only keep a weak reference to the core here
        let to_notify = Arc::downgrade(&self.core);

        // Monitor for changes (see below for the implementation against to_notify's type)
        let lifetime = to_monitor.when_changed_if_unchanged(Arc::new(to_notify));
        let already_changed = lifetime.is_none();

        // Store the lifetime
        let mut last_notification = lifetime;
        mem::swap(&mut last_notification, &mut core.existing_notification);

        // Any lifetime that was in the core before this one should be finished
        last_notification.map(|mut last_notification| last_notification.done());

        // Return if the value is already changed
        already_changed
    }
}

///
/// The weak reference to a core is generated in `monitor_changes`: this specifies what happens when a
/// notification is generated for such a reference.
///
impl<Value, TFn> Notifiable for Weak<Mutex<ComputedBindingCore<Value, TFn>>>
where
    Value: 'static + Clone + Send,
    TFn: 'static + Send + Sync + Fn() -> Value,
{
    fn mark_as_changed(&self) {
        // If the reference is still active, reconstitute a computed binding in order to call the mark_changed method
        if let Some(to_notify) = self.upgrade() {
            let to_notify = ComputedBinding { core: to_notify };
            to_notify.mark_changed(false);
        } else if cfg!(debug_assertions) {
            // We can carry on here, but this suggests a memory leak - if the core has gone, then its owning object should have stopped this event from firing
            panic!("The core of a computed is gone but its notifcations have been left behind");
        }
    }
}

impl<Value: 'static + Clone + Send, TFn> Clone for ComputedBinding<Value, TFn>
where
    TFn: 'static + Send + Sync + Fn() -> Value,
{
    fn clone(&self) -> Self {
        ComputedBinding {
            core: Arc::clone(&self.core),
        }
    }
}

impl<Value: 'static + Clone, TFn> Changeable for ComputedBinding<Value, TFn>
where
    TFn: 'static + Send + Sync + Fn() -> Value,
{
    fn when_changed(&self, what: Arc<dyn Notifiable>) -> Box<dyn Releasable> {
        let releasable = ReleasableNotifiable::new(what);

        // Lock the core and push this as a thing to perform when this value changes
        let mut core = self.core.lock().unwrap();
        core.when_changed.push(releasable.clone_as_owned());

        core.filter_unused_notifications();

        Box::new(releasable)
    }
}

impl<Value: 'static + Clone + Send, TFn> Bound<Value> for ComputedBinding<Value, TFn>
where
    TFn: 'static + Send + Sync + Fn() -> Value,
{
    fn get(&self) -> Value {
        // This is a dependency of the current binding context
        BindingContext::add_dependency(self.clone());

        // Set to true if the value changes while we're reading it
        // (presumably because it's updating rapidly)
        let mut notify_immediately = false;
        let result;

        {
            // Borrow the core
            let mut core = self.core.lock().unwrap();

            if let Cached(value) = core.get() {
                // The value already exists in this item
                result = value;
            } else {
                // TODO: really want to recalculate without locking the core - can do this by moving the function out and doing the recalculation here
                // TODO: locking the core and calling a function can result in deadlocks due to user code structure in particular against other bindings
                // TODO: when we do recalculate without locking, we need to make sure that no extra invalidations arrived between when we started the calculation and when we stored the result
                // TODO: if multiple calculations do occur outside the lock, we need to return only the most recent result so when_changed is fired correctly

                // Stop responding to notifications
                let mut old_notification = None;
                mem::swap(&mut old_notification, &mut core.existing_notification);

                if let Some(mut last_notification) = old_notification {
                    last_notification.done();
                }

                // Need to re-calculate the core
                let (value, mut dependencies) = core.recalculate();

                // If any of the dependencies change, mark this item as changed too
                notify_immediately = self.monitor_changes(&mut core, &mut dependencies);

                // If we're going to notify, unset the value we've cached
                if notify_immediately {
                    core.latest_value = ComputedValue::Unknown;
                }

                // TODO: also need to make sure that any hooks we have are removed if we're only referenced via a hook

                // Return the value
                result = value;
            }
        }

        // If there was a change while we were calculating the value, generate a notification
        if notify_immediately {
            self.mark_changed(true);
        }

        result
    }
}
