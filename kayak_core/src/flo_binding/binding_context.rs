use super::traits::*;
use super::notify_fn::*;

use std::rc::*;
use std::sync::*;
use std::cell::*;

thread_local! {
    static CURRENT_CONTEXT: RefCell<Option<BindingContext>> = RefCell::new(None);
}

///
/// Represents the dependencies of a binding context
///
#[derive(Clone)]
pub struct BindingDependencies {
    /// Set to true if the binding dependencies have been changed since they were registered in the dependencies
    recently_changed: Arc<Mutex<bool>>,

    /// The when_changed monitors for the recently_changed flag
    recent_change_monitors: Rc<RefCell<Vec<Box<dyn Releasable>>>>,

    /// The list of changables that are dependent on this context
    dependencies: Rc<RefCell<Vec<Box<dyn Changeable>>>>
}

impl BindingDependencies {
    ///
    /// Creates a new binding dependencies object
    ///
    pub fn new() -> BindingDependencies {
        BindingDependencies {
            recently_changed:       Arc::new(Mutex::new(false)),
            recent_change_monitors: Rc::new(RefCell::new(vec![])),
            dependencies:           Rc::new(RefCell::new(vec![]))
        }
    }

    ///
    /// Adds a new dependency to this object
    ///
    pub fn add_dependency<TChangeable: Changeable+'static>(&mut self, dependency: TChangeable) {
        // Set the recently changed flag so that we can tell if the dependencies are already out of date before when_changed is called
        let recently_changed            = Arc::clone(&self.recently_changed);
        let mut recent_change_monitors  = self.recent_change_monitors.borrow_mut();
        recent_change_monitors.push(dependency.when_changed(notify(move || { *recently_changed.lock().unwrap() = true; })));

        // Add this dependency to the list
        self.dependencies.borrow_mut().push(Box::new(dependency))
    }

    ///
    /// If the dependencies have not changed since they were registered, registers for changes
    /// and returns a `Releasable`. If the dependencies are already different, returns `None`.
    /// 
    pub fn when_changed_if_unchanged(&self, what: Arc<dyn Notifiable>) -> Option<Box<dyn Releasable>> {
        let mut to_release = vec![];

        // Register with all of the dependencies
        for dep in self.dependencies.borrow_mut().iter_mut() {
            to_release.push(dep.when_changed(Arc::clone(&what)));
        }

        if *self.recently_changed.lock().unwrap() {
            // If a value changed while we were building these dependencies, then immediately generate the notification
            to_release.into_iter().for_each(|mut releasable| releasable.done());

            // Nothing to release
            None
        } else {
            // Otherwise, return the set of releasable values
            Some(Box::new(to_release))
        }
    }
}

impl Changeable for BindingDependencies {
    fn when_changed(&self, what: Arc<dyn Notifiable>) -> Box<dyn Releasable> {
        let when_changed_or_not = self.when_changed_if_unchanged(Arc::clone(&what));

        match when_changed_or_not {
            Some(releasable)    => releasable,
            None                => {
                what.mark_as_changed();
                Box::new(vec![])
            }
        }
    }
}

///
/// Represents a binding context. Binding contexts are
/// per-thread structures, used to track 
///
#[derive(Clone)]
pub struct BindingContext {
    /// The dependencies for this context
    dependencies: BindingDependencies,

    /// None, or the binding context that this context was created within
    nested: Option<Box<BindingContext>>
}

impl BindingContext {
    ///
    /// Gets the active binding context
    ///
    pub fn current() -> Option<BindingContext> {
        CURRENT_CONTEXT.with(|current_context| {
            current_context
                .borrow()
                .as_ref()
                .cloned()
        })
    }

    ///
    /// Panics if we're trying to create a binding, with a particular message
    /// 
    pub fn panic_if_in_binding_context(msg: &str) {
        if CURRENT_CONTEXT.with(|context| context.borrow().is_some()) {
            panic!("Not possible when binding: {}", msg);
        }
    }

    ///
    /// Executes a function in a new binding context
    ///
    pub fn bind<TResult, TFn>(to_do: TFn) -> (TResult, BindingDependencies) 
    where TFn: FnOnce() -> TResult {
        // Remember the previous context
        let previous_context = Self::current();

        // Create a new context
        let dependencies    = BindingDependencies::new();
        let new_context     = BindingContext {
            dependencies:   dependencies.clone(),
            nested:         previous_context.clone().map(Box::new)
        };

        // Make the current context the same as the new context
        CURRENT_CONTEXT.with(|current_context| *current_context.borrow_mut() = Some(new_context));

        // Perform the requested action with this context
        let result = to_do();

        // Reset to the previous context
        CURRENT_CONTEXT.with(|current_context| *current_context.borrow_mut() = previous_context);

        (result, dependencies)
    }

    #[allow(dead_code)]
    ///
    /// Performs an action outside of the binding context (dependencies 
    /// will not be tracked for anything the supplied function does)
    ///
    pub fn out_of_context<TResult, TFn>(to_do: TFn) -> TResult
    where TFn: FnOnce() -> TResult {
        // Remember the previous context
        let previous_context = Self::current();

        // Unset the context
        CURRENT_CONTEXT.with(|current_context| *current_context.borrow_mut() = None);

        // Perform the operations without a binding context
        let result = to_do();

        // Reset to the previous context
        CURRENT_CONTEXT.with(|current_context| *current_context.borrow_mut() = previous_context);

        result
    }

    ///
    /// Adds a dependency to the current context (if one is found)
    /// 
    pub fn add_dependency<TChangeable: Changeable+'static>(dependency: TChangeable) {
        Self::current().map(|mut ctx| ctx.dependencies.add_dependency(dependency));
    }
}
